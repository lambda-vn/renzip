//! Reading and writing of Ren'Py .rpa archives.
//!
//! An .rpa file is a container, not a compressed format: file data is stored
//! raw and only the index at the end is deflated. Extraction is therefore a
//! seek and a read, and this crate never holds more than one entry in memory.
//!
//! Everything an archive says about itself is treated as hostile input. Paths
//! are checked before they reach the file system ([`path`]), byte ranges are
//! checked against the real file size, and the index is parsed rather than
//! interpreted, so no pickle opcode can construct anything.
//!
//! The crate knows nothing about Tauri or about any user interface, so it can
//! be tested and reused on its own.
//!
//! ```no_run
//! # fn main() -> Result<(), renzip_core::Error> {
//! let mut archive = renzip_core::Archive::open("game/archive.rpa")?;
//! println!("{} entries", archive.entries().len());
//! let report = archive.extract(
//!     renzip_core::Selection::All,
//!     std::path::Path::new("out"),
//!     renzip_core::Layout::Tree,
//!     &mut |_progress| true,
//! )?;
//! println!("{} files written", report.extracted);
//! # Ok(())
//! # }
//! ```

#![deny(missing_docs)]
#![forbid(unsafe_code)]

pub mod archive;
pub mod builder;
pub mod error;
pub mod header;
pub mod index;
pub mod path;
pub mod reason;

pub use archive::{Archive, Entry, ExtractionReport, Layout, Progress, Rejection, Selection};
pub use builder::{build, scan, BuildReport, ScanReport, SourceFile, DEFAULT_KEY};
pub use error::{Error, IndexProblem, Result, Truncation};
pub use header::Version;
pub use reason::{IoAction, Reason};
