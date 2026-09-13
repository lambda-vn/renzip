//! Building an .rpa archive from a directory tree.
//!
//! The layout written here is the one Ren'Py's own archiver produces: a fixed
//! width header line, then every file stored back to back and uncompressed,
//! then the deflated pickle of the index. Because the header has a fixed
//! width, it is written once with a placeholder offset and rewritten in place
//! once the index has found its home.

use std::fs::{File, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::archive::{Progress, Rejection};
use crate::error::{Error, Result};
use crate::header::{self, Version};
use crate::index::{self, RawEntry};
use crate::path as safe_path;
use crate::reason::{IoAction, Reason};

/// Key written into an RPA-3.0 header when the caller has no opinion.
///
/// The value is arbitrary: the key obfuscates offsets, it protects nothing.
/// It is rpatool's default, not Ren'Py's own archiver's - that one always
/// uses `0x42424242` - kept here as this crate's own default rather than
/// changed to match, since nothing reads either value as meaningful.
pub const DEFAULT_KEY: u64 = 0xDEAD_BEEF;

/// One file found under a source directory.
#[derive(Debug, Clone, Serialize)]
pub struct SourceFile {
    /// Name the file would get inside the archive, separators normalized.
    pub path: String,
    /// Size of the file on disk.
    pub size: u64,
}

/// What a walk of a source directory found.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ScanReport {
    /// Files that would be packed, sorted by archive name.
    pub files: Vec<SourceFile>,
    /// Files left out, with the reason for each.
    pub skipped: Vec<Rejection>,
}

/// Outcome of a build.
#[derive(Debug, Clone, Default, Serialize)]
pub struct BuildReport {
    /// Number of files stored.
    pub entries: usize,
    /// Total size of the stored file data.
    pub bytes: u64,
    /// Files left out of the archive, with the reason for each.
    pub skipped: Vec<Rejection>,
    /// True when the caller asked to stop before the end.
    pub cancelled: bool,
}

/// Walk a source directory and list what a build would pack.
///
/// Exists so that an interface can show the content of a folder, and its
/// size, before committing to writing an archive out of it. [`build`] runs the
/// same walk, so what is listed here is what gets packed.
///
/// # Errors
///
/// [`Error::InvalidSource`] when the directory cannot be read.
pub fn scan(source_dir: &Path) -> Result<ScanReport> {
    if !source_dir.is_dir() {
        return Err(Error::InvalidSource {
            path: source_dir.to_path_buf(),
            error: None,
        });
    }

    let mut report = ScanReport::default();
    collect_files(
        source_dir,
        source_dir,
        &mut report.files,
        &mut report.skipped,
    )?;
    report.files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(report)
}

/// Pack every file under `source_dir` into a new archive at `dest_path`.
///
/// `excludes` names files or folders below `source_dir` (each an archive
/// path, with no trailing separator on a folder) to leave out, the way the
/// creation screen lets a user drop something they staged by mistake, at
/// any depth, before committing to a build.
///
/// The archive is written to a temporary file next to `dest_path` and only
/// moved into place once it is complete, so a build that fails partway, or
/// that finds `dest_path` inside `source_dir` and would otherwise read the
/// file it is writing, never touches whatever was at `dest_path` before.
///
/// `on_progress` is called once per stored file and stops the build when it
/// returns `false`; a cancelled build discards its temporary file and leaves
/// `dest_path` exactly as it was (untouched if nothing was there yet).
///
/// # Errors
///
/// Fails when the source directory cannot be walked, the destination cannot
/// be written, or `version` is one this crate cannot write (only RPA-2.0 and
/// RPA-3.0 can be written). On any error after the temporary file was
/// created, it is removed and `dest_path` is left exactly as it was.
pub fn build(
    source_dir: &Path,
    dest_path: &Path,
    version: Version,
    key: u64,
    excludes: &[String],
    on_progress: &mut dyn FnMut(&Progress<'_>) -> bool,
) -> Result<BuildReport> {
    // Fail on an unwritable version before touching the file system, so that
    // a bad request never leaves a truncated archive behind.
    let placeholder = header::format_line(version, 0, key)?;
    let key = if version == Version::Rpa2_0 { 0 } else { key };

    let temp_path = temp_path_for(dest_path);
    // Neither the destination this build would replace nor the temporary
    // file it writes through belong inside the archive being built.
    let reserved = [&temp_path, dest_path]
        .into_iter()
        .filter_map(canonical_prospective)
        .collect::<Vec<_>>();

    let scanned = scan(source_dir)?;
    let canonical_source = source_dir
        .canonicalize()
        .map_err(|e| Error::io(source_dir, e))?;
    let mut report = BuildReport::default();
    let mut files = Vec::with_capacity(scanned.files.len());
    for file in scanned.files {
        if is_excluded(&file.path, excludes) {
            continue;
        }
        let native = file.path.replace('/', std::path::MAIN_SEPARATOR_STR);
        if reserved.contains(&canonical_source.join(native)) {
            report.skipped.push(Rejection {
                path: file.path,
                reason: Reason::DestinationOfBuild,
            });
            continue;
        }
        files.push(file);
    }
    report.skipped.extend(scanned.skipped);

    let outcome = write_temp_archive(
        &temp_path,
        source_dir,
        files,
        version,
        key,
        &placeholder,
        report,
        on_progress,
    );

    let report = match outcome {
        Ok(report) => report,
        Err(e) => {
            let _ = std::fs::remove_file(&temp_path);
            return Err(e);
        }
    };

    if report.cancelled {
        let _ = std::fs::remove_file(&temp_path);
    } else {
        std::fs::rename(&temp_path, dest_path).map_err(|e| {
            let _ = std::fs::remove_file(&temp_path);
            Error::io(dest_path, e)
        })?;
    }

    Ok(report)
}

/// Write every file into `temp_path`, starting from `placeholder` as the
/// header. Does not touch `dest_path`: [`build`] moves the temporary file
/// into place once this returns `Ok` with `cancelled` false.
#[allow(clippy::too_many_arguments)]
fn write_temp_archive(
    temp_path: &Path,
    source_dir: &Path,
    files: Vec<SourceFile>,
    version: Version,
    key: u64,
    placeholder: &str,
    mut report: BuildReport,
    on_progress: &mut dyn FnMut(&Progress<'_>) -> bool,
) -> Result<BuildReport> {
    let mut out = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(temp_path)
        .map_err(|e| Error::io(temp_path, e))?;
    out.write_all(placeholder.as_bytes())
        .map_err(|e| Error::io(temp_path, e))?;

    let total = files.len();
    let bytes_total: u64 = files.iter().map(|file| file.size).sum();
    let mut offset = placeholder.len() as u64;
    let mut entries = Vec::with_capacity(files.len());

    for (done, file) in files.into_iter().enumerate() {
        let name = file.path;
        // The scan named every file relative to the root, so the path on disk
        // is that name below the root again.
        let path = source_dir.join(&name);
        match store_file(&mut out, temp_path, &path, offset) {
            Ok(written) => {
                entries.push(RawEntry {
                    path: name.clone(),
                    segments: vec![(offset, written)],
                    prefix: Vec::new(),
                });
                offset += written;
                report.entries += 1;
                report.bytes += written;
            }
            Err(reason) => report.skipped.push(Rejection {
                path: name.clone(),
                reason,
            }),
        }

        let keep_going = on_progress(&Progress {
            done: done + 1,
            total,
            bytes_done: report.bytes,
            bytes_total,
            current: &name,
        });
        if !keep_going {
            // Nothing here will be moved into place: report that no file was
            // actually stored, rather than counting work about to be thrown
            // away.
            report.cancelled = true;
            report.entries = 0;
            report.bytes = 0;
            return Ok(report);
        }
    }

    let index_offset = offset;
    let encoded = index::encode(&entries, key)?;
    out.write_all(&encoded)
        .map_err(|e| Error::io(temp_path, e))?;

    let final_header = header::format_line(version, index_offset, key)?;
    debug_assert_eq!(final_header.len(), placeholder.len());
    out.seek(SeekFrom::Start(0))
        .map_err(|e| Error::io(temp_path, e))?;
    out.write_all(final_header.as_bytes())
        .map_err(|e| Error::io(temp_path, e))?;
    out.flush().map_err(|e| Error::io(temp_path, e))?;

    Ok(report)
}

/// Name of the temporary file `build` writes through before moving it to
/// `dest_path`: a sibling file, so the final move is a same-volume rename.
fn temp_path_for(dest_path: &Path) -> PathBuf {
    let mut name = dest_path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    dest_path.with_file_name(name)
}

/// Canonical path `path` resolves to, even when `path` itself does not exist
/// yet: only its parent directory needs to, which lets a not-yet-created
/// destination or temporary file still be compared against the canonical
/// paths a directory scan produces. `None` when the parent cannot be
/// resolved, in which case nothing is excluded on `path`'s account.
fn canonical_prospective(path: &Path) -> Option<PathBuf> {
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty())?;
    let canonical_parent = parent.canonicalize().ok()?;
    let file_name = path.file_name()?;
    Some(canonical_parent.join(file_name))
}

/// Walk `dir` and collect one [`SourceFile`] per file found.
///
/// Symbolic links are skipped rather than followed: a link is the one way a
/// directory tree can pull in content from outside itself or loop forever,
/// and neither belongs in an archive built from that tree. A `.rpa` or
/// `.rpi` file is skipped too, at any depth: see [`is_archive_file`].
fn collect_files(
    root: &Path,
    dir: &Path,
    files: &mut Vec<SourceFile>,
    skipped: &mut Vec<Rejection>,
) -> Result<()> {
    let listing = std::fs::read_dir(dir).map_err(|e| Error::InvalidSource {
        path: dir.to_path_buf(),
        error: Some(e.to_string()),
    })?;

    for item in listing {
        let item = item.map_err(|e| Error::InvalidSource {
            path: dir.to_path_buf(),
            error: Some(e.to_string()),
        })?;
        let path = item.path();
        let file_type = item.file_type().map_err(|e| Error::InvalidSource {
            path: path.clone(),
            error: Some(e.to_string()),
        })?;
        let name = archive_name(root, &path);

        if file_type.is_symlink() {
            skipped.push(Rejection {
                path: name,
                reason: Reason::SymbolicLink,
            });
        } else if file_type.is_dir() {
            collect_files(root, &path, files, skipped)?;
        } else if is_archive_file(&path) {
            skipped.push(Rejection {
                path: name,
                reason: Reason::ArchiveInArchive,
            });
        } else if let Err(rejection) = safe_path::sanitize(&name) {
            skipped.push(Rejection {
                path: name,
                reason: Reason::UnusableName { rejection },
            });
        } else {
            let size = item
                .metadata()
                .map_err(|e| Error::InvalidSource {
                    path: path.clone(),
                    error: Some(e.to_string()),
                })?
                .len();
            files.push(SourceFile { path: name, size });
        }
    }
    Ok(())
}

/// Whether `path` names a `.rpa` or `.rpi` file, extension compared without
/// regard to case.
///
/// Ren'Py only ever turns a `.rpa`/`.rpi` sitting directly on disk into an
/// archive (`scandirfiles_from_filesystem` in `renpy/loader.py`, SDK 8.5.3
/// around lines 431-454); it never looks inside one archive for another.
/// Packing either into an archive built here would only ever be dead weight
/// the engine cannot reach, so neither is ever worth including.
fn is_archive_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("rpa") || ext.eq_ignore_ascii_case("rpi"))
}

/// Whether `path` falls under one of the `excludes`, at any depth.
///
/// An exclude matches a whole file (`"readme.txt"`) or everything below a
/// folder (`"images"` drops `"images/bg.png"`, and `"images/ui"` drops only
/// what sits under that nested folder), never a partial name, so `"image"`
/// never drops `"images/bg.png"`.
fn is_excluded(path: &str, excludes: &[String]) -> bool {
    excludes
        .iter()
        .any(|excluded| path == excluded || path.starts_with(&format!("{excluded}/")))
}

/// Name a file gets inside the archive: its path below the root, with '/'.
fn archive_name(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

/// Copy one file into the archive at `offset`, returning the bytes written.
///
/// `archive` is the path `out` was opened from, named when moving inside it
/// fails. A file that cannot be read is reported rather than fatal: the size
/// was read before the copy, and a file can be locked or removed in between.
fn store_file(
    out: &mut File,
    archive: &Path,
    path: &Path,
    offset: u64,
) -> std::result::Result<u64, Reason> {
    let mut input = File::open(path).map_err(|e| Reason::io(IoAction::Read, path.display(), e))?;
    out.seek(SeekFrom::Start(offset))
        .map_err(|e| Reason::io(IoAction::Seek, archive.display(), e))?;
    std::io::copy(&mut input, out).map_err(|e| Reason::io(IoAction::Store, path.display(), e))
}
