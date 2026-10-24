//! Opening, listing and extracting an .rpa archive.

use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{Error, Result, Truncation};
use crate::header::{self, Header, Version, MAX_HEADER_LEN};
use crate::index::{self, RawEntry};
use crate::path as safe_path;
use crate::reason::{IoAction, Reason};

/// Largest buffer reserved up front when reading an entry into memory.
const READ_CAPACITY_HINT: usize = 64 * 1024 * 1024;

/// One file stored in an archive, after every validation has passed.
#[derive(Debug, Clone, Serialize)]
pub struct Entry {
    /// Path inside the archive, separators normalized to '/'.
    pub path: String,
    /// Offset of the first segment's stored bytes inside the archive file.
    ///
    /// The one detail shown to a user; an entry made of several segments
    /// (see [`Entry::segments`]) has no single meaningful offset, so this is
    /// where reading it begins.
    pub offset: u64,
    /// Size of the file once extracted, prefix and every segment included.
    pub length: u64,
    /// Bytes stored in the index that open the file content.
    #[serde(skip)]
    pub prefix: Vec<u8>,
    /// Byte ranges to read from the archive and concatenate, in order, to
    /// get the file content after `prefix`. Almost always a single range;
    /// more than one when the index listed several `(offset, length)` tuples
    /// for this entry, which Ren'Py itself reads by concatenating them.
    #[serde(skip)]
    pub segments: Vec<(u64, u64)>,
}

/// An entry the archive announced but that will not be extracted.
#[derive(Debug, Clone, Serialize)]
pub struct Rejection {
    /// Path as the archive spelled it.
    pub path: String,
    /// Why it was refused: a code an interface translates, whose `Display`
    /// is an English sentence.
    pub reason: Reason,
}

/// What an extraction is allowed to touch.
#[derive(Debug, Clone, Copy)]
pub enum Selection<'a> {
    /// Every entry of the archive.
    All,
    /// Only these paths, as returned by [`Archive::entries`].
    Paths(&'a [String]),
}

/// How extracted files are laid out under the destination directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Deserialize)]
pub enum Layout {
    /// Keep the folders the archive paths describe.
    #[default]
    Tree,
    /// Drop every folder and write file names side by side.
    ///
    /// Two entries can then claim the same name; the second one is reported as
    /// skipped rather than silently overwriting the first, since losing a file
    /// to a name collision is exactly the kind of quiet data loss an extractor
    /// must never do.
    Flat,
}

/// State of an extraction, handed to the progress callback.
#[derive(Debug, Clone, Copy)]
pub struct Progress<'a> {
    /// Entries finished so far.
    pub done: usize,
    /// Entries the extraction will go through.
    pub total: usize,
    /// Bytes written so far.
    pub bytes_done: u64,
    /// Bytes the extraction will write if it runs to the end.
    pub bytes_total: u64,
    /// Entry that was just finished.
    pub current: &'a str,
}

/// Outcome of an extraction.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ExtractionReport {
    /// Number of files written.
    pub extracted: usize,
    /// Total bytes written.
    pub bytes: u64,
    /// Entries left out, with the reason for each.
    pub skipped: Vec<Rejection>,
    /// True when the caller asked to stop before the end.
    pub cancelled: bool,
}

/// A read-only view of one .rpa archive and its open file handle.
pub struct Archive {
    path: PathBuf,
    file: File,
    file_size: u64,
    version: Version,
    entries: Vec<Entry>,
    by_path: HashMap<String, usize>,
    rejected: Vec<Rejection>,
}

impl std::fmt::Debug for Archive {
    /// Summarize the archive rather than dumping every entry of it.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Archive")
            .field("path", &self.path)
            .field("version", &self.version)
            .field("file_size", &self.file_size)
            .field("entries", &self.entries.len())
            .field("rejected", &self.rejected.len())
            .finish()
    }
}

impl Archive {
    /// Open an archive and decode its index.
    ///
    /// Entries whose path or byte range cannot be trusted are dropped here
    /// rather than at extraction time, so that a hostile path never reaches
    /// the file system layer at all; they stay visible through
    /// [`Archive::rejected`].
    ///
    /// # Errors
    ///
    /// Fails when the file cannot be read, its header is not a supported
    /// version, or its index is missing or malformed.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let mut file = File::open(&path).map_err(|e| Error::io(&path, e))?;
        let file_size = file.metadata().map_err(|e| Error::io(&path, e))?.len();

        let header = read_header(&mut file, &path)?;
        let raw_entries = match header.version {
            Version::Rpa1_0 => {
                let index_path = path.with_extension("rpi");
                if !index_path.is_file() {
                    return Err(Error::MissingIndexFile { path: index_path });
                }
                let index_file = File::open(&index_path).map_err(|e| Error::io(&index_path, e))?;
                index::decode(index_file, 0)?
            }
            _ => {
                if header.index_offset >= file_size {
                    return Err(Error::Truncated {
                        truncation: Truncation::IndexPastEnd {
                            offset: header.index_offset,
                            size: file_size,
                        },
                    });
                }
                file.seek(SeekFrom::Start(header.index_offset))
                    .map_err(|e| Error::io(&path, e))?;
                index::decode(&mut file, header.key)?
            }
        };

        let (entries, rejected) = validate(raw_entries, file_size);
        let by_path = entries
            .iter()
            .enumerate()
            .map(|(i, entry)| (entry.path.clone(), i))
            .collect();

        Ok(Archive {
            path,
            file,
            file_size,
            version: header.version,
            entries,
            by_path,
            rejected,
        })
    }

    /// Path the archive was opened from.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Version announced by the header.
    pub fn version(&self) -> Version {
        self.version
    }

    /// Size of the archive file itself, in bytes.
    pub fn file_size(&self) -> u64 {
        self.file_size
    }

    /// Every usable entry, sorted by path.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Entries the archive announced but that were refused, with the reason.
    pub fn rejected(&self) -> &[Rejection] {
        &self.rejected
    }

    /// Total extracted size of every usable entry.
    pub fn total_size(&self) -> u64 {
        self.entries
            .iter()
            .fold(0, |total, entry| u64::saturating_add(total, entry.length))
    }

    /// Read one entry into memory.
    ///
    /// Callers decide what size they are willing to hold: this is meant for
    /// previews and small files, extraction streams to disk instead.
    ///
    /// # Errors
    ///
    /// [`Error::EntryNotFound`] for an unknown path, [`Error::Truncated`] when
    /// the file ends inside the entry.
    pub fn read_entry(&mut self, path: &str) -> Result<Vec<u8>> {
        let index = *self.by_path.get(path).ok_or_else(|| Error::EntryNotFound {
            entry: path.to_string(),
        })?;
        let entry = self.entries[index].clone();

        // Only a hint: a forged index can announce far more than the file
        // holds (the same range listed again and again), and the buffer grows
        // by what is actually read.
        let hint = usize::try_from(entry.length)
            .unwrap_or(usize::MAX)
            .min(READ_CAPACITY_HINT);
        let mut buffer = Vec::with_capacity(hint);
        buffer.extend_from_slice(&entry.prefix);
        for (i, &(offset, length)) in entry.segments.iter().enumerate() {
            // The first segment's declared length still includes the prefix
            // when there is one (only ever true for a single-segment entry);
            // every later segment is read in full.
            let read_len = if i == 0 {
                length - entry.prefix.len() as u64
            } else {
                length
            };
            self.file
                .seek(SeekFrom::Start(offset))
                .map_err(|e| Error::io(&self.path, e))?;
            (&mut self.file)
                .take(read_len)
                .read_to_end(&mut buffer)
                .map_err(|e| Error::io(&self.path, e))?;
        }

        if buffer.len() as u64 != entry.length {
            return Err(Error::Truncated {
                truncation: Truncation::EntryStopsEarly {
                    read: buffer.len() as u64,
                    length: entry.length,
                    entry: entry.path,
                },
            });
        }
        Ok(buffer)
    }

    /// Extract a selection of entries into `dest_dir`.
    ///
    /// `dest_dir` is created if needed. `on_progress` is called once per
    /// finished entry and stops the extraction when it returns `false`; the
    /// files already written are kept and the report says it was cancelled.
    ///
    /// An entry that cannot be written, or whose destination turns out to sit
    /// outside `dest_dir` once symbolic links are resolved, is reported in
    /// [`ExtractionReport::skipped`] and does not abort the rest.
    ///
    /// # Errors
    ///
    /// Fails only for problems that concern the whole run: an unusable
    /// destination directory, or a path in `Selection::Paths` that no entry
    /// carries.
    pub fn extract(
        &mut self,
        selection: Selection<'_>,
        dest_dir: &Path,
        layout: Layout,
        on_progress: &mut dyn FnMut(&Progress<'_>) -> bool,
    ) -> Result<ExtractionReport> {
        let indices = self.resolve_selection(selection)?;
        let root = prepare_destination(dest_dir)?;
        let mut taken: HashSet<PathBuf> = HashSet::with_capacity(indices.len());

        let mut report = ExtractionReport {
            extracted: 0,
            bytes: 0,
            skipped: Vec::new(),
            cancelled: false,
        };
        let total = indices.len();
        let bytes_total: u64 = indices.iter().map(|&i| self.entries[i].length).sum();

        // Split the borrow so that reading through the file handle and looking
        // at the entry list can happen in the same loop.
        let Archive {
            file,
            entries,
            path,
            ..
        } = self;

        for (done, &i) in indices.iter().enumerate() {
            let entry = &entries[i];
            match write_entry(file, path, entry, &root, layout, &mut taken) {
                Ok(written) => {
                    report.extracted += 1;
                    report.bytes += written;
                }
                Err(reason) => report.skipped.push(Rejection {
                    path: entry.path.clone(),
                    reason,
                }),
            }

            let keep_going = on_progress(&Progress {
                done: done + 1,
                total,
                bytes_done: report.bytes,
                bytes_total,
                current: &entry.path,
            });
            if !keep_going {
                report.cancelled = true;
                break;
            }
        }
        Ok(report)
    }

    /// Turn a selection into entry indices, in listing order.
    fn resolve_selection(&self, selection: Selection<'_>) -> Result<Vec<usize>> {
        match selection {
            Selection::All => Ok((0..self.entries.len()).collect()),
            Selection::Paths(paths) => {
                let mut indices = Vec::with_capacity(paths.len());
                for path in paths {
                    let index = *self.by_path.get(path).ok_or_else(|| Error::EntryNotFound {
                        entry: path.clone(),
                    })?;
                    indices.push(index);
                }
                indices.sort_unstable();
                indices.dedup();
                Ok(indices)
            }
        }
    }
}

/// Read the header line, falling back to RPA-1.0 detection.
///
/// An RPA-1.0 archive has no header at all: it starts with file data and
/// keeps its index in a sibling .rpi file, so a file that does not announce
/// itself is treated as one only when that sibling exists.
fn read_header(file: &mut File, path: &Path) -> Result<Header> {
    let mut buffer = [0u8; MAX_HEADER_LEN];
    let read = file.read(&mut buffer).map_err(|e| Error::io(path, e))?;
    let window = &buffer[..read];

    if window.starts_with(b"RPA-") {
        if let Some(end) = window.iter().position(|&b| b == b'\n') {
            let line = String::from_utf8_lossy(&window[..end]);
            return header::parse_line(line.trim_end_matches('\r'));
        }
    }
    if path.with_extension("rpi").is_file() {
        return Ok(Header {
            version: Version::Rpa1_0,
            index_offset: 0,
            key: 0,
        });
    }
    Err(Error::UnsupportedHeader {
        header: String::from_utf8_lossy(&window[..read.min(32)]).to_string(),
    })
}

/// Check every raw entry against the path rules and the archive bounds.
///
/// Returns the usable entries sorted by path, and the refused ones.
fn validate(raw_entries: Vec<RawEntry>, file_size: u64) -> (Vec<Entry>, Vec<Rejection>) {
    let mut entries = Vec::with_capacity(raw_entries.len());
    let mut rejected = Vec::new();
    let mut seen: HashMap<String, ()> = HashMap::with_capacity(raw_entries.len());

    for raw in raw_entries {
        if raw.segments.is_empty() {
            rejected.push(Rejection {
                path: raw.path,
                reason: Reason::UnreadableSegments,
            });
            continue;
        }
        if let Err(rejection) = safe_path::sanitize(&raw.path) {
            rejected.push(Rejection {
                path: raw.path,
                reason: Reason::UnsafePath { rejection },
            });
            continue;
        }
        if let Some(reason) = out_of_bounds(&raw, file_size) {
            rejected.push(Rejection {
                path: raw.path,
                reason,
            });
            continue;
        }

        let normalized = raw.path.replace('\\', "/");
        if seen.insert(normalized.clone(), ()).is_some() {
            rejected.push(Rejection {
                path: raw.path,
                reason: Reason::DuplicatePath { path: normalized },
            });
            continue;
        }
        // A single segment's declared length already includes the prefix
        // (out_of_bounds just checked it fits); several segments never carry
        // one, so their declared lengths sum to the whole entry directly.
        let length = if raw.segments.len() == 1 {
            raw.segments[0].1
        } else {
            raw.segments.iter().map(|&(_, len)| len).sum()
        };
        entries.push(Entry {
            path: normalized,
            offset: raw.segments[0].0,
            length,
            prefix: raw.prefix,
            segments: raw.segments,
        });
    }

    entries.sort_by(|a, b| a.path.cmp(&b.path));
    (entries, rejected)
}

/// Describe why a byte range cannot be read, if that is the case.
///
/// A range whose end does not even fit in a `u64` is out of bounds too: the
/// header key is XORed into offsets and lengths, and a forged key can push
/// both past 2^63, where adding them overflows instead of landing past the
/// end of the file.
fn out_of_bounds(raw: &RawEntry, file_size: u64) -> Option<Reason> {
    if raw.segments.len() == 1 {
        let (offset, length) = raw.segments[0];
        let prefix_len = raw.prefix.len() as u64;
        if prefix_len > length {
            return Some(Reason::PrefixTooLong {
                prefix: prefix_len,
                length,
            });
        }
        let stored = length - prefix_len;
        let Some(end) = offset.checked_add(stored) else {
            return Some(Reason::RangeOverflow {
                offset,
                length: stored,
            });
        };
        if end > file_size {
            return Some(Reason::PastEnd {
                start: offset,
                end,
                size: file_size,
            });
        }
        return None;
    }

    // Several segments, concatenated with no prefix: each is checked against
    // the archive on its own, the way Ren'Py reads each one on its own.
    // The same range may be listed any number of times, so the entry's total
    // length can exceed the file and must itself fit in a `u64`.
    let mut total: u64 = 0;
    for &(offset, length) in &raw.segments {
        let Some(end) = offset.checked_add(length) else {
            return Some(Reason::SegmentOverflow { offset, length });
        };
        if end > file_size {
            return Some(Reason::SegmentPastEnd {
                start: offset,
                end,
                size: file_size,
            });
        }
        let Some(sum) = total.checked_add(length) else {
            return Some(Reason::SegmentOverflow { offset, length });
        };
        total = sum;
    }
    None
}

/// Create the destination directory and return its canonical form.
fn prepare_destination(dest_dir: &Path) -> Result<PathBuf> {
    std::fs::create_dir_all(dest_dir).map_err(|e| Error::InvalidDestination {
        path: dest_dir.to_path_buf(),
        error: Some(e.to_string()),
    })?;
    let root = dest_dir
        .canonicalize()
        .map_err(|e| Error::InvalidDestination {
            path: dest_dir.to_path_buf(),
            error: Some(e.to_string()),
        })?;
    if !root.is_dir() {
        return Err(Error::InvalidDestination {
            path: root,
            error: None,
        });
    }
    Ok(root)
}

/// Write one entry under `root`, streaming its bytes.
///
/// Returns the number of bytes written, or the reason the entry was skipped.
/// Skipping is deliberately not an [`Error`]: one unwritable file out of fifty
/// thousand must not throw away the extraction that already succeeded.
fn write_entry(
    file: &mut File,
    archive: &Path,
    entry: &Entry,
    root: &Path,
    layout: Layout,
    taken: &mut HashSet<PathBuf>,
) -> std::result::Result<u64, Reason> {
    let mut relative =
        safe_path::sanitize(&entry.path).map_err(|rejection| Reason::UnsafePath { rejection })?;
    if layout == Layout::Flat {
        relative = PathBuf::from(relative.file_name().ok_or(Reason::NoFileName)?);
    }
    if !taken.insert(relative.clone()) {
        return Err(Reason::AlreadyWritten {
            path: relative.display().to_string(),
        });
    }
    let target = root.join(&relative);
    let parent = target.parent().ok_or(Reason::NoParentDirectory)?;

    std::fs::create_dir_all(parent)
        .map_err(|e| Reason::io(IoAction::Create, parent.display(), e))?;
    // The path itself was checked before any of this, but a symbolic link
    // planted inside the destination could still point out of it, so the
    // directory that will really be written to is checked once it exists.
    match safe_path::is_contained(root, parent) {
        Ok(true) => {}
        Ok(false) => {
            return Err(Reason::OutsideDestination {
                root: root.display().to_string(),
            })
        }
        Err(e) => return Err(Reason::io(IoAction::Resolve, parent.display(), e)),
    }
    // Same reasoning for the file: opening it would follow a link that was
    // there before us, so an existing link is removed rather than written
    // through. A plain file is left to be overwritten as usual.
    if let Ok(metadata) = std::fs::symlink_metadata(&target) {
        if metadata.file_type().is_symlink() {
            std::fs::remove_file(&target)
                .map_err(|e| Reason::io(IoAction::ReplaceLink, target.display(), e))?;
        }
    }

    let mut out = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(&target)
        .map_err(|e| Reason::io(IoAction::Write, target.display(), e))?;

    out.write_all(&entry.prefix)
        .map_err(|e| Reason::io(IoAction::Write, target.display(), e))?;
    let mut copied = 0u64;
    for (i, &(offset, length)) in entry.segments.iter().enumerate() {
        // The first segment's declared length still includes the prefix
        // when there is one (only ever true for a single-segment entry);
        // every later segment is copied in full.
        let read_len = if i == 0 {
            length - entry.prefix.len() as u64
        } else {
            length
        };
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| Reason::io(IoAction::Read, archive.display(), e))?;
        copied += std::io::copy(&mut file.take(read_len), &mut out)
            .map_err(|e| Reason::io(IoAction::Extract, &entry.path, e))?;
    }
    out.flush()
        .map_err(|e| Reason::io(IoAction::Write, target.display(), e))?;

    let written = copied + entry.prefix.len() as u64;
    if written != entry.length {
        return Err(Reason::ArchiveEndsEarly {
            written,
            announced: entry.length,
        });
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn raw(segments: Vec<(u64, u64)>) -> RawEntry {
        RawEntry {
            path: "a.txt".into(),
            segments,
            prefix: Vec::new(),
        }
    }

    #[test]
    fn a_range_whose_end_overflows_is_out_of_bounds() {
        // A forged header key can XOR offsets and lengths past 2^63, where
        // their sum no longer fits in a u64.
        let huge = u64::MAX - 1;
        assert!(out_of_bounds(&raw(vec![(huge, 10)]), 1000).is_some());
        assert!(out_of_bounds(&raw(vec![(0, 4), (huge, 10)]), 1000).is_some());
    }

    #[test]
    fn segments_whose_lengths_overflow_together_are_out_of_bounds() {
        // Each segment fits the file; listed twice, they no longer fit a u64.
        let half = 1u64 << 63;
        assert!(out_of_bounds(&raw(vec![(0, half), (0, half)]), half).is_some());
    }

    #[test]
    fn a_range_inside_the_file_is_accepted() {
        assert!(out_of_bounds(&raw(vec![(10, 5)]), 15).is_none());
        assert!(out_of_bounds(&raw(vec![(10, 5), (0, 3)]), 15).is_none());
    }
}
