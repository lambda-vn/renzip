//! Error type shared by every operation of the crate.

use std::path::PathBuf;

/// Anything that can go wrong while reading or writing an .rpa archive.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The underlying file could not be read or written.
    #[error("I/O error on {path}: {source}")]
    Io {
        /// File the operation was working on.
        path: PathBuf,
        /// Original OS error.
        source: std::io::Error,
    },

    /// The first line of the file is not a header this crate understands.
    #[error("unsupported archive header: {0}")]
    UnsupportedHeader(String),

    /// An RPA-1.0 archive was detected but its .rpi index file is missing.
    #[error("missing index file {0} for this RPA-1.0 archive")]
    MissingIndexFile(PathBuf),

    /// The index could not be decompressed, unpickled, or has an unexpected shape.
    #[error("corrupt archive index: {0}")]
    CorruptIndex(String),

    /// The archive claims an index or an entry that reaches past its own end.
    #[error("archive is truncated: {0}")]
    Truncated(String),

    /// No entry with this path exists in the archive.
    #[error("no such entry in archive: {0}")]
    EntryNotFound(String),

    /// The destination directory of an extraction is unusable.
    #[error("invalid destination directory {path}: {reason}")]
    InvalidDestination {
        /// Directory that was rejected.
        path: PathBuf,
        /// Why it was rejected.
        reason: String,
    },

    /// A source directory handed to the builder is unusable.
    #[error("invalid source directory {path}: {reason}")]
    InvalidSource {
        /// Directory that was rejected.
        path: PathBuf,
        /// Why it was rejected.
        reason: String,
    },

    /// The requested archive version cannot be written by this crate.
    #[error("cannot write archives in version {0}")]
    UnsupportedWriteVersion(&'static str),
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
