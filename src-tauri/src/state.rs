//! Open archives, kept between commands.
//!
//! A session holds the path and the entry list, not the archive itself: every
//! operation reopens the file on its own thread. Decoding an index again
//! costs milliseconds, while holding one lock across a multi-gigabyte
//! extraction would freeze every other command for the duration.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use renzip_core::{Archive, Entry};

use crate::error::CommandError;

/// One archive the window has open.
pub struct Session {
    /// Path the archive was opened from.
    pub path: PathBuf,
    /// Entry list, as decoded when it was opened.
    pub entries: Arc<Vec<Entry>>,
    /// Stops the extraction running on this archive.
    pub cancel: CancelSlot,
    /// Scratch directory the row menu writes copies into, one per archive.
    pub scratch: PathBuf,
}

impl Session {
    /// Build a session from an archive that was just opened.
    pub fn new(path: PathBuf, archive: Archive) -> Self {
        Session {
            path,
            entries: Arc::new(archive.entries().to_vec()),
            cancel: CancelSlot::default(),
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
    /// Stops the running build.
    pub build_cancel: CancelSlot,
}

impl AppState {
    /// Register a session and return its handle.
    pub fn insert(&self, session: Session) -> u32 {
        let handle = self.next_handle.fetch_add(1, Ordering::Relaxed) + 1;
        self.lock().insert(handle, Arc::new(session));
        handle
    }

    /// Look a session up.
    ///
    /// # Errors
    ///
    /// [`CommandError::ArchiveNotOpen`] when the handle is unknown, which
    /// means the archive was closed while a command was in flight.
    pub fn session(&self, handle: u32) -> Result<Arc<Session>, CommandError> {
        self.lock()
            .get(&handle)
            .cloned()
            .ok_or(CommandError::ArchiveNotOpen { handle })
    }

    /// Drop a session, cancelling anything still running on it and taking its
    /// scratch copies with it.
    pub fn remove(&self, handle: u32) {
        // Out of the registry first, and the lock released: deleting the
        // scratch directory can take a while, and every other command waits
        // on that lock meanwhile.
        let removed = self.lock().remove(&handle);
        if let Some(session) = removed {
            session.cancel.cancel();
            session.discard_scratch();
        }
    }

    /// Drop every session, on the way out of the application.
    ///
    /// Closing the window is the other way an archive stops being open, and
    /// the copies the row menu made have no reason to outlive it.
    pub fn close_all(&self) {
        let sessions = std::mem::take(&mut *self.lock());
        for session in sessions.into_values() {
            session.cancel.cancel();
            session.discard_scratch();
        }
    }

    /// Lock the session registry, recovering it if a prior panic poisoned it.
    ///
    /// A poisoned lock here only means some earlier command panicked while
    /// holding it; the map itself is still a valid map, so refusing every
    /// later command over that is worse than carrying on with it.
    fn lock(&self) -> MutexGuard<'_, HashMap<u32, Arc<Session>>> {
        self.sessions.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// The stop flag of the latest operation of one kind.
///
/// Every operation gets a flag of its own. Resetting one shared flag when an
/// operation starts would wipe a stop already asked for, including the one
/// closing the archive sets a moment before.
#[derive(Default)]
pub struct CancelSlot(Mutex<Arc<AtomicBool>>);

impl CancelSlot {
    /// Give a new operation its flag, which [`CancelSlot::cancel`] sets from
    /// now on.
    pub fn start(&self) -> Arc<AtomicBool> {
        let flag = Arc::new(AtomicBool::new(false));
        *self.lock() = Arc::clone(&flag);
        flag
    }

    /// Stop the latest operation.
    pub fn cancel(&self) {
        self.lock().store(true, Ordering::Relaxed);
    }

    /// Lock the slot, recovering it the same way as [`AppState::lock`].
    fn lock(&self) -> MutexGuard<'_, Arc<AtomicBool>> {
        self.0.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use super::{scratch_dir, CancelSlot};

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

    #[test]
    fn a_stop_reaches_the_latest_operation_and_never_the_next() {
        let slot = CancelSlot::default();
        let first = slot.start();
        slot.cancel();
        assert!(first.load(Ordering::Relaxed));

        let second = slot.start();
        assert!(
            !second.load(Ordering::Relaxed),
            "a stop asked for the previous operation must not cancel this one"
        );
        slot.cancel();
        assert!(second.load(Ordering::Relaxed));
    }
}
