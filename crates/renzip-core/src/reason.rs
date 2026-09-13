//! Why a file was left out or refused, as data an interface can translate.
//!
//! Reports used to carry an English sentence, which a window in another
//! language could only show as is. A [`Reason`] is a code and the details
//! that code needs instead: it serializes as `{"code": "...", ...details}`
//! for an interface to word in its own language, and its `Display` still
//! gives the English sentence, for logs and for callers with no translation
//! of their own.

use serde::Serialize;

use crate::path::PathRejection;

/// What a failed operation on the file system was doing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IoAction {
    /// Creating a directory.
    Create,
    /// Resolving a directory to its canonical path.
    Resolve,
    /// Removing a symbolic link that stood where a file was to be written.
    ReplaceLink,
    /// Writing a file.
    Write,
    /// Reading a file.
    Read,
    /// Copying an entry's bytes out of the archive.
    Extract,
    /// Moving to an offset inside the archive being written.
    Seek,
    /// Copying a source file into the archive being written.
    Store,
}

/// Why a file was left out of an extraction or a build, or refused when an
/// archive was opened.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum Reason {
    /// The index lists several tuples for the entry and at least one is not
    /// a plain `(offset, length)` pair, which Ren'Py cannot read either.
    UnreadableSegments,
    /// The entry path cannot be trusted.
    UnsafePath {
        /// What is wrong with the path.
        rejection: PathRejection,
    },
    /// A single-segment entry declares a prefix longer than the entry.
    PrefixTooLong {
        /// Size of the prefix, in bytes.
        prefix: u64,
        /// Size of the whole entry, in bytes.
        length: u64,
    },
    /// The entry's bytes run past the end of the archive.
    PastEnd {
        /// First byte of the range.
        start: u64,
        /// Byte just past the end of the range.
        end: u64,
        /// Size of the archive, in bytes.
        size: u64,
    },
    /// One segment of the entry runs past the end of the archive.
    SegmentPastEnd {
        /// First byte of the segment.
        start: u64,
        /// Byte just past the end of the segment.
        end: u64,
        /// Size of the archive, in bytes.
        size: u64,
    },
    /// The entry's range ends further than any file offset can reach.
    RangeOverflow {
        /// Offset the range starts at.
        offset: u64,
        /// Bytes the range claims.
        length: u64,
    },
    /// One segment of the entry ends further than any file offset can reach.
    SegmentOverflow {
        /// Offset the segment starts at.
        offset: u64,
        /// Bytes the segment claims.
        length: u64,
    },
    /// Another entry of the archive already has this path.
    DuplicatePath {
        /// The path both entries share, separators normalized.
        path: String,
    },
    /// A flat extraction found no file name to write the entry under.
    NoFileName,
    /// The entry resolves to no parent directory to write into.
    NoParentDirectory,
    /// Another entry of the same extraction already wrote this file.
    AlreadyWritten {
        /// The file both entries would write, relative to the destination.
        path: String,
    },
    /// Once symbolic links are resolved, the file would land outside the
    /// destination.
    OutsideDestination {
        /// The destination directory.
        root: String,
    },
    /// The archive holds fewer bytes for the entry than its index announces.
    ArchiveEndsEarly {
        /// Bytes actually written.
        written: u64,
        /// Bytes the index announced.
        announced: u64,
    },
    /// The file is the archive the build is writing.
    DestinationOfBuild,
    /// The file is a symbolic link, which is never followed.
    SymbolicLink,
    /// The file is a `.rpa` or `.rpi` archive, which Ren'Py never opens from
    /// inside another archive.
    ArchiveInArchive,
    /// The file name cannot be stored in an archive.
    UnusableName {
        /// What is wrong with the name.
        rejection: PathRejection,
    },
    /// An operation on the file system failed.
    Io {
        /// What the operation was doing.
        action: IoAction,
        /// File or directory it was working on.
        path: String,
        /// The operating system's own message.
        error: String,
    },
}

impl Reason {
    /// Build a [`Reason::Io`] for `action` failing on `path`.
    pub(crate) fn io(
        action: IoAction,
        path: impl std::fmt::Display,
        error: std::io::Error,
    ) -> Self {
        Reason::Io {
            action,
            path: path.to_string(),
            error: error.to_string(),
        }
    }
}

impl std::fmt::Display for Reason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Reason::UnreadableSegments => write!(
                f,
                "index lists several tuples for this entry and at least one is not \
                 a plain (offset, length) pair; Ren'Py cannot read this entry either"
            ),
            Reason::UnsafePath { rejection } => write!(f, "unsafe path: {rejection}"),
            Reason::PrefixTooLong { prefix, length } => {
                write!(
                    f,
                    "declares a {prefix} byte prefix inside a {length} byte entry"
                )
            }
            Reason::PastEnd { start, end, size } => {
                write!(f, "declares bytes {start}..{end} of a {size} byte archive")
            }
            Reason::SegmentPastEnd { start, end, size } => write!(
                f,
                "declares a segment at bytes {start}..{end} of a {size} byte archive"
            ),
            Reason::RangeOverflow { offset, length } => write!(
                f,
                "declares {length} bytes at offset {offset}, past the end of any file"
            ),
            Reason::SegmentOverflow { offset, length } => write!(
                f,
                "declares a segment of {length} bytes at offset {offset}, past the end of any file"
            ),
            Reason::DuplicatePath { path } => {
                write!(f, "duplicate path, '{path}' is already in the archive")
            }
            Reason::NoFileName => write!(f, "entry has no file name"),
            Reason::NoParentDirectory => write!(f, "entry has no parent directory"),
            Reason::AlreadyWritten { path } => {
                write!(f, "another entry of this extraction already wrote {path}")
            }
            Reason::OutsideDestination { root } => write!(f, "would write outside of {root}"),
            Reason::ArchiveEndsEarly { written, announced } => write!(
                f,
                "archive ends after {written} of the {announced} bytes announced"
            ),
            Reason::DestinationOfBuild => {
                write!(f, "left out: this is the destination of this build")
            }
            Reason::SymbolicLink => write!(f, "symbolic link"),
            Reason::ArchiveInArchive => write!(
                f,
                "archive file: Ren'Py never opens one packed inside another"
            ),
            Reason::UnusableName { rejection } => write!(f, "unusable name: {rejection}"),
            Reason::Io {
                action,
                path,
                error,
            } => match action {
                IoAction::Create => write!(f, "cannot create {path}: {error}"),
                IoAction::Resolve => write!(f, "cannot resolve {path}: {error}"),
                IoAction::ReplaceLink => write!(f, "cannot replace the link at {path}: {error}"),
                IoAction::Write => write!(f, "cannot write {path}: {error}"),
                IoAction::Read => write!(f, "cannot read {path}: {error}"),
                IoAction::Extract => write!(f, "cannot extract {path}: {error}"),
                IoAction::Seek => write!(f, "cannot seek in {path}: {error}"),
                IoAction::Store => write!(f, "cannot store {path}: {error}"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn to_json(reason: &Reason) -> serde_json::Value {
        serde_json::to_value(reason).unwrap()
    }

    #[test]
    fn serializes_as_a_code_and_its_details() {
        assert_eq!(
            to_json(&Reason::SymbolicLink),
            json!({"code": "symbolic_link"})
        );
        assert_eq!(
            to_json(&Reason::PastEnd {
                start: 1,
                end: 9,
                size: 5
            }),
            json!({"code": "past_end", "start": 1, "end": 9, "size": 5})
        );
        assert_eq!(
            to_json(&Reason::UnsafePath {
                rejection: PathRejection::Traversal
            }),
            json!({"code": "unsafe_path", "rejection": {"kind": "traversal"}})
        );
        assert_eq!(
            to_json(&Reason::UnusableName {
                rejection: PathRejection::ReservedName("CON".into())
            }),
            json!({"code": "unusable_name", "rejection": {"kind": "reserved_name", "detail": "CON"}})
        );
        assert_eq!(
            to_json(&Reason::Io {
                action: IoAction::ReplaceLink,
                path: "a".into(),
                error: "denied".into()
            }),
            json!({"code": "io", "action": "replace_link", "path": "a", "error": "denied"})
        );
    }

    #[test]
    fn displays_an_english_sentence() {
        assert_eq!(
            Reason::UnsafePath {
                rejection: PathRejection::Traversal
            }
            .to_string(),
            "unsafe path: path traversal"
        );
        assert_eq!(
            Reason::Io {
                action: IoAction::Write,
                path: "out/a.txt".into(),
                error: "denied".into()
            }
            .to_string(),
            "cannot write out/a.txt: denied"
        );
    }
}
