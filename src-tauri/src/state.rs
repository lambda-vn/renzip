//! Open archives, kept between commands.
//!
//! A session holds the path and the entry list, not the archive itself: every
//! operation reopens the file on its own thread. Decoding an index again
//! costs milliseconds, while holding one lock across a multi-gigabyte
//! extraction would freeze every other command for the duration.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use renzip_core::{Archive, Entry};

/// One archive the window has open.
pub struct Session {
    /// Path the archive was opened from.
    pub path: PathBuf,
    /// Entry list, as decoded when it was opened.
    pub entries: Arc<Vec<Entry>>,
    /// Set to true to stop the extraction running on this archive.
    pub cancel: Arc<AtomicBool>,
    /// Scratch directory the row menu writes copies into, one per archive.
    pub scratch: PathBuf,
}

impl Session {
    /// Build a session from an archive that was just opened.
    pub fn new(path: PathBuf, archive: Archive) -> Self {
        Session {
            path,
            entries: Arc::new(archive.entries().to_vec()),
            cancel: Arc::new(AtomicBool::new(false)),
            scratch: scratch_dir(),
        }
    }

    /// Remove the scratch directory and everything the menu put in it.
    ///
    /// Failure is not worth reporting: the directory belongs to the system's
    /// temporary folder, which is cleaned up eventually anyway, and a file
    /// still open in the application the user just started is exactly the
    /// case where deleting it should not be insisted on.
    pub fn discard_scratch(&self) {
        let _ = std::fs::remove_dir_all(&self.scratch);
    }
}

/// Pick a scratch directory no other archive, and no other run, will claim.
///
/// Two archives can hold the same script.rpyc, and one overwriting the
/// other's copy would open the wrong file. The process id keeps two running
/// windows apart, and the clock keeps two archives of the same window apart.
fn scratch_dir() -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_nanos())
        .unwrap_or(0);
    std::env::temp_dir()
        .join("renzip")
        .join(format!("{:x}-{:x}", std::process::id(), nanos))
}

/// Every open archive, keyed by the handle the window holds.
#[derive(Default)]
pub struct AppState {
    sessions: Mutex<HashMap<u32, Arc<Session>>>,
    next_handle: AtomicU32,
    /// Set to true to stop the running build.
    pub build_cancel: Arc<AtomicBool>,
}

impl AppState {
    /// Register a session and return its handle.
    pub fn insert(&self, session: Session) -> u32 {
        let handle = self.next_handle.fetch_add(1, Ordering::Relaxed) + 1;
        self.sessions
            .lock()
            .expect("archive registry")
            .insert(handle, Arc::new(session));
        handle
    }

    /// Look a session up.
    ///
    /// # Errors
    ///
    /// Returns a message for the window when the handle is unknown, which
    /// means the archive was closed while a command was in flight.
    pub fn session(&self, handle: u32) -> Result<Arc<Session>, String> {
        self.sessions
            .lock()
            .expect("archive registry")
            .get(&handle)
            .cloned()
            .ok_or_else(|| format!("archive {handle} is not open"))
    }

    /// Drop a session, cancelling anything still running on it and taking its
    /// scratch copies with it.
    pub fn remove(&self, handle: u32) {
        if let Some(session) = self
            .sessions
            .lock()
            .expect("archive registry")
            .remove(&handle)
        {
            session.cancel.store(true, Ordering::Relaxed);
            session.discard_scratch();
        }
    }

    /// Drop every session, on the way out of the application.
    ///
    /// Closing the window is the other way an archive stops being open, and
    /// the copies the row menu made have no reason to outlive it.
    pub fn close_all(&self) {
        for (_, session) in self.sessions.lock().expect("archive registry").drain() {
            session.cancel.store(true, Ordering::Relaxed);
            session.discard_scratch();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::scratch_dir;

    #[test]
    fn two_archives_never_share_a_scratch_directory() {
        let first = scratch_dir();
        let second = scratch_dir();
        assert_ne!(
            first, second,
            "the same folder for two archives means the copy of one script.rpyc overwrites the other"
        );
        assert!(first.starts_with(std::env::temp_dir()));
        assert!(first
            .parent()
            .is_some_and(|parent| parent.ends_with("renzip")));
    }
}
