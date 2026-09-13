//! Errors a command hands back to the window.
//!
//! The window words every error itself, in the language it is set to, so
//! what crosses the IPC boundary is a code and its details rather than a
//! sentence. Errors from renzip-core go through with the codes they already
//! carry; this layer only adds the few that it alone can raise.

use serde::Serialize;

use renzip_core::Reason;

/// Why a command failed.
#[derive(Debug, Serialize)]
#[serde(tag = "code", rename_all = "snake_case")]
pub enum CommandError {
    /// The archive was closed while a command about it was on its way.
    ArchiveNotOpen {
        /// Handle the window still held.
        handle: u32,
    },
    /// The one entry an "open" extracts was left out.
    EntrySkipped {
        /// Path of the entry inside the archive.
        entry: String,
        /// Why it was left out.
        reason: Reason,
    },
    /// The window asked for a version this application does not write.
    UnsupportedWriteVersion {
        /// The version asked for.
        version: String,
    },
    /// The desktop could not show the file in its folder.
    RevealFailed {
        /// The opener's own message.
        error: String,
    },
    /// A background task ended before it could report anything.
    Internal {
        /// The runtime's own message.
        error: String,
    },
    /// An error of renzip-core, sent with the code it already carries.
    #[serde(untagged)]
    Core(renzip_core::Error),
}

impl From<renzip_core::Error> for CommandError {
    fn from(error: renzip_core::Error) -> Self {
        CommandError::Core(error)
    }
}

impl From<tauri::Error> for CommandError {
    fn from(error: tauri::Error) -> Self {
        CommandError::Internal {
            error: error.to_string(),
        }
    }
}

impl From<tauri_plugin_opener::Error> for CommandError {
    fn from(error: tauri_plugin_opener::Error) -> Self {
        CommandError::RevealFailed {
            error: error.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_as_a_code_and_its_details() {
        assert_eq!(
            serde_json::to_value(CommandError::ArchiveNotOpen { handle: 3 }).unwrap(),
            json!({"code": "archive_not_open", "handle": 3})
        );
        assert_eq!(
            serde_json::to_value(CommandError::EntrySkipped {
                entry: "a.txt".into(),
                reason: Reason::SymbolicLink,
            })
            .unwrap(),
            json!({"code": "entry_skipped", "entry": "a.txt", "reason": {"code": "symbolic_link"}})
        );
    }

    #[test]
    fn a_core_error_keeps_its_own_code() {
        let error = CommandError::from(renzip_core::Error::EntryNotFound {
            entry: "a.txt".into(),
        });
        assert_eq!(
            serde_json::to_value(error).unwrap(),
            json!({"code": "entry_not_found", "entry": "a.txt"})
        );
    }
}
