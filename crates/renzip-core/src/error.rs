//! Error type shared by every operation of the crate.
//!
//! Like a [`crate::Reason`], an error is data before it is a sentence: it
//! serializes as `{"code": "...", ...details}` for an interface to word in its
//! own language, and its `Display` is the English sentence. Only what the
//! operating system or a library says (an I/O message, a zlib or pickle
//! error) is passed on as text.

use std::fmt::Display;
use std::path::{Path, PathBuf};

use serde::{Serialize, Serializer};

/// Anything that can go wrong while reading or writing an .rpa archive.
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum Error {
    /// The underlying file could not be read or written.
    #[error("I/O error on {path}: {source}")]
    Io {
        /// File the operation was working on.
        #[serde(serialize_with = "as_path")]
        path: PathBuf,
        /// Original OS error.
        #[serde(rename = "error", serialize_with = "as_text")]
        source: std::io::Error,
    },

    /// The first line of the file is not a header this crate understands.
    #[error("unsupported archive header: {header}")]
    UnsupportedHeader {
        /// The start of the file, read as text.
        header: String,
    },

    /// An RPA-1.0 archive was detected but its .rpi index file is missing.
    #[error("missing index file {path} for this RPA-1.0 archive")]
    MissingIndexFile {
        /// Where the index file was looked for.
        #[serde(serialize_with = "as_path")]
        path: PathBuf,
    },

    /// The index could not be decompressed, unpickled, or has an unexpected shape.
    #[error("corrupt archive index: {problem}")]
    CorruptIndex {
        /// What is wrong with it.
        problem: IndexProblem,
    },

    /// The archive claims an index or an entry that reaches past its own end.
    #[error("archive is truncated: {truncation}")]
    Truncated {
        /// Where the file ends too early.
        truncation: Truncation,
    },

    /// No entry with this path exists in the archive.
    #[error("no such entry in archive: {entry}")]
    EntryNotFound {
        /// The path that was asked for.
        entry: String,
    },

    /// The destination directory of an extraction is unusable.
    #[error("invalid destination directory {path}: {}", .error.as_deref().unwrap_or("not a directory"))]
    InvalidDestination {
        /// Directory that was rejected.
        #[serde(serialize_with = "as_path")]
        path: PathBuf,
        /// The operating system's message, or `None` when the path exists
        /// but is not a directory.
        error: Option<String>,
    },

    /// A source directory handed to the builder is unusable.
    #[error("invalid source directory {path}: {}", .error.as_deref().unwrap_or("not a directory"))]
    InvalidSource {
        /// Directory that was rejected.
        #[serde(serialize_with = "as_path")]
        path: PathBuf,
        /// The operating system's message, or `None` when the path is not a
        /// directory.
        error: Option<String>,
    },

    /// The requested archive version cannot be written by this crate.
    #[error("cannot write archives in version {version}")]
    UnsupportedWriteVersion {
        /// The version that was asked for.
        version: &'static str,
    },
}

/// What is wrong with an archive index.
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum IndexProblem {
    /// The zlib stream does not inflate.
    #[error("cannot inflate the index: {error}")]
    Inflate {
        /// The decompressor's own message.
        error: String,
    },
    /// The inflated index is larger than [`crate::index::MAX_INDEX_BYTES`].
    #[error("index inflates past the {limit} byte limit")]
    TooLarge {
        /// The limit, in bytes.
        limit: u64,
    },
    /// The inflated bytes are not a pickle.
    #[error("cannot unpickle the index: {error}")]
    Unpickle {
        /// The unpickler's own message.
        error: String,
    },
    /// The pickle holds something other than a dict.
    #[error("index is not a dict")]
    NotADict,
    /// A key of the dict is not a string.
    #[error("index key is not a string: {key}")]
    KeyNotAString {
        /// The key, as the unpickler describes it.
        key: String,
    },
    /// An entry is not a list of tuples.
    #[error("entry '{entry}' is not a list of tuples")]
    NotAList {
        /// Name of the entry.
        entry: String,
    },
    /// An entry lists no tuple at all.
    #[error("entry '{entry}' has no tuple")]
    NoTuple {
        /// Name of the entry.
        entry: String,
    },
    /// An entry's only tuple has neither 2 nor 3 fields.
    #[error("entry '{entry}' has {count} fields, expected 2 or 3")]
    FieldCount {
        /// Name of the entry.
        entry: String,
        /// How many fields the tuple has.
        count: usize,
    },
    /// An entry is described by something that is not a tuple.
    #[error("entry '{entry}' is not described by a tuple")]
    NotATuple {
        /// Name of the entry.
        entry: String,
    },
    /// An entry's prefix is not a byte string.
    #[error("entry '{entry}' has a prefix that is not a byte string: {value}")]
    PrefixNotBytes {
        /// Name of the entry.
        entry: String,
        /// The prefix, as the unpickler describes it.
        value: String,
    },
    /// An entry's offset or length is not a positive integer.
    #[error("entry '{entry}' has an offset or length that is not a positive integer: {value}")]
    NotAPositiveInteger {
        /// Name of the entry.
        entry: String,
        /// The value, as the unpickler describes it.
        value: String,
    },
    /// Pickling a new index failed.
    #[error("cannot pickle the index: {error}")]
    Pickle {
        /// The pickler's own message.
        error: String,
    },
    /// Deflating a new index failed.
    #[error("cannot deflate the index: {error}")]
    Deflate {
        /// The compressor's own message.
        error: String,
    },
}

/// Where an archive ends before what it announces.
#[derive(Debug, thiserror::Error, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Truncation {
    /// The header places the index at or past the end of the file.
    #[error("the index is announced at offset {offset} of a {size} byte file")]
    IndexPastEnd {
        /// Offset the header announces.
        offset: u64,
        /// Size of the file, in bytes.
        size: u64,
    },
    /// An entry's bytes stop before its announced length.
    #[error("entry '{entry}' stops after {read} of {length} bytes")]
    EntryStopsEarly {
        /// Name of the entry.
        entry: String,
        /// Bytes that could be read.
        read: u64,
        /// Bytes the index announces.
        length: u64,
    },
}

/// Result alias used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Build an [`Error::Io`] carrying the path the failure happened on.
    ///
    /// A bare `std::io::Error` never says which file it came from, and every
    /// message this crate produces ends up in front of a user who only knows
    /// their archive by its name.
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Error::Io {
            path: path.into(),
            source,
        }
    }
}

/// Serialize a value as the text it displays as.
fn as_text<T: Display, S: Serializer>(
    value: &T,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    serializer.collect_str(value)
}

/// Serialize a path as text, replacing whatever is not valid Unicode.
fn as_path<P: AsRef<Path>, S: Serializer>(
    path: &P,
    serializer: S,
) -> std::result::Result<S::Ok, S::Error> {
    serializer.collect_str(&path.as_ref().display())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn to_json(error: &Error) -> serde_json::Value {
        serde_json::to_value(error).unwrap()
    }

    #[test]
    fn serializes_as_a_code_and_its_details() {
        assert_eq!(
            to_json(&Error::io("game/a.rpa", std::io::Error::other("denied"))),
            json!({"code": "io", "path": "game/a.rpa", "error": "denied"})
        );
        assert_eq!(
            to_json(&Error::CorruptIndex {
                problem: IndexProblem::FieldCount {
                    entry: "a.rpy".into(),
                    count: 4
                }
            }),
            json!({"code": "corrupt_index", "problem": {"kind": "field_count", "entry": "a.rpy", "count": 4}})
        );
        assert_eq!(
            to_json(&Error::InvalidDestination {
                path: "out".into(),
                error: None
            }),
            json!({"code": "invalid_destination", "path": "out", "error": null})
        );
        assert_eq!(
            to_json(&Error::UnsupportedWriteVersion { version: "RPA-3.2" }),
            json!({"code": "unsupported_write_version", "version": "RPA-3.2"})
        );
    }

    #[test]
    fn displays_the_english_sentence() {
        assert_eq!(
            Error::InvalidDestination {
                path: "out".into(),
                error: None
            }
            .to_string(),
            "invalid destination directory out: not a directory"
        );
        assert_eq!(
            Error::Truncated {
                truncation: Truncation::IndexPastEnd { offset: 9, size: 5 }
            }
            .to_string(),
            "archive is truncated: the index is announced at offset 9 of a 5 byte file"
        );
    }
}
