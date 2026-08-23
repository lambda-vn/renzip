//! Application layer: the commands the window calls and the events it listens to.
//!
//! Every archive operation lives in renzip-core; this module only holds the
//! open archives, moves long operations off the UI thread, and turns core
//! types into something the frontend can read.

mod desktop;
mod preview;
mod state;

use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;

use renzip_core::{Archive, Layout, Progress, Rejection, Selection, Version};

use crate::preview::Preview;
use crate::state::{AppState, Session};

/// Event name every progress update is emitted under.
const PROGRESS_EVENT: &str = "renzip://progress";

/// Shortest delay between two progress events.
///
/// An archive holds tens of thousands of entries and each event crosses the
/// IPC boundary; at one event per entry the window spends its time decoding
/// JSON instead of drawing. The last event of a run is always sent.
const PROGRESS_INTERVAL: Duration = Duration::from_millis(60);

/// What the window is told about a freshly opened archive.
#[derive(Debug, Serialize)]
pub struct ArchiveSummary {
    /// Handle to pass back to every other command.
    handle: u32,
    /// File name of the archive.
    name: String,
    /// Full path of the archive.
    path: String,
    /// Version announced by its header.
    version: String,
    /// Number of usable entries.
    entry_count: usize,
    /// Total size of those entries once extracted.
    total_size: u64,
    /// Size of the archive file itself.
    file_size: u64,
    /// Entries refused at open time, with the reason for each.
    rejected: Vec<Rejection>,
    /// Entries whose index held more than one tuple.
    multi_segment: Vec<String>,
}

/// One row of the entry list.
#[derive(Debug, Serialize)]
pub struct EntryRow {
    /// Path inside the archive.
    path: String,
    /// Size once extracted.
    size: u64,
    /// Offset in the archive, shown in the details panel.
    offset: u64,
    /// Bytes of this entry that are stored in the index rather than in the
    /// data area, shown in the details panel because a non-zero prefix is the
    /// one thing that makes an entry read differently from a plain slice.
    prefix_len: usize,
}

/// What opening an entry ended up doing.
#[derive(Debug, Serialize)]
pub struct OpenOutcome {
    /// Where the temporary copy was written.
    path: String,
    /// Which of the three things happened, so the window can say it.
    action: OpenAction,
}

/// The three ways handing a file to the desktop can end.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum OpenAction {
    /// An application claimed the file and was started.
    Opened,
    /// Nothing claimed it, so the desktop's "open with" chooser was shown.
    Chooser,
    /// Nothing claimed it and this desktop has no chooser, so the file was
    /// shown in its folder.
    Revealed,
}

/// Progress of a running extraction or build.
#[derive(Debug, Clone, Serialize)]
struct ProgressEvent {
    /// Which operation is running: "extract" or "build".
    phase: &'static str,
    /// Archive handle the operation belongs to, 0 for a build.
    handle: u32,
    /// Files finished so far.
    done: usize,
    /// Files the operation will go through.
    total: usize,
    /// Bytes written so far.
    bytes_done: u64,
    /// Bytes the operation will write if it runs to the end.
    bytes_total: u64,
    /// File that was just finished.
    current: String,
}

/// Open an archive and register it under a fresh handle.
#[tauri::command]
async fn open_archive(state: State<'_, AppState>, path: String) -> Result<ArchiveSummary, String> {
    let path = PathBuf::from(path);
    let archive = tauri::async_runtime::spawn_blocking({
        let path = path.clone();
        move || Archive::open(path)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    // Everything the window is told is read before the archive moves into the
    // session that owns it from here on.
    let summary = ArchiveSummary {
        handle: 0,
        name: file_name(&path),
        path: path.display().to_string(),
        version: archive.version().as_str().to_string(),
        entry_count: archive.entries().len(),
        total_size: archive.total_size(),
        file_size: archive.file_size(),
        rejected: archive.rejected().to_vec(),
        multi_segment: archive.multi_segment_paths().to_vec(),
    };

    let handle = state.insert(Session::new(path, archive));
    Ok(ArchiveSummary { handle, ..summary })
}

/// List every usable entry of an open archive.
#[tauri::command]
fn list_entries(state: State<'_, AppState>, handle: u32) -> Result<Vec<EntryRow>, String> {
    let session = state.session(handle)?;
    Ok(session
        .entries
        .iter()
        .map(|entry| EntryRow {
            path: entry.path.clone(),
            size: entry.length,
            offset: entry.offset,
            prefix_len: entry.prefix.len(),
        })
        .collect())
}

/// List what packing a directory would put into an archive.
#[tauri::command]
async fn scan_directory(source_dir: String) -> Result<renzip_core::ScanReport, String> {
    tauri::async_runtime::spawn_blocking(move || {
        renzip_core::scan(Path::new(&source_dir)).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Say what the desktop would do with a file carrying this extension.
///
/// The row menu asks before it opens, so that it can drop its "open with"
/// entry when there is no registered application for it to differ from.
#[tauri::command]
fn entry_association(extension: String) -> String {
    let probe = std::path::PathBuf::from(format!("probe.{extension}"));
    match desktop::association_of(&probe) {
        desktop::Association::Runnable => "runnable",
        desktop::Association::Picker => "picker",
        desktop::Association::Dead => "dead",
        desktop::Association::Unknown => "unknown",
    }
    .to_string()
}

/// Forget an archive and release its handle.
#[tauri::command]
fn close_archive(state: State<'_, AppState>, handle: u32) {
    state.remove(handle);
}

/// Read one entry and decide how the window should show it.
#[tauri::command]
async fn preview_entry(
    state: State<'_, AppState>,
    handle: u32,
    path: String,
) -> Result<Preview, String> {
    let session = state.session(handle)?;
    let size = session
        .entries
        .iter()
        .find(|entry| entry.path == path)
        .map(|entry| entry.length)
        .ok_or_else(|| format!("no such entry in archive: {path}"))?;

    if let Some(refusal) = preview::too_big(&path, size) {
        return Ok(refusal);
    }

    let archive_path = session.path.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut archive = Archive::open(&archive_path).map_err(|e| e.to_string())?;
        let bytes = archive.read_entry(&path).map_err(|e| e.to_string())?;
        Ok(preview::of(&path, bytes))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Extract one entry to a temporary directory and hand it to the desktop.
///
/// Backs the row menu's "open": the file has to exist somewhere before any
/// application can be pointed at it. The directory is the system temporary
/// one, under a folder of ours, so nothing is written where the user keeps
/// their own files.
///
/// The whole thing lives here rather than in the window because the path is
/// then never chosen by the frontend: it can ask to open entry X of archive
/// Y, and nothing else. No command taking an arbitrary path to open is
/// exposed at all.
///
/// `chooser` skips the application registered for the type and goes straight
/// to the desktop's picker. Without it, an extension someone once pointed at
/// an editor always lands in that editor, with no way to say otherwise.
#[tauri::command]
async fn open_entry(
    app: AppHandle,
    state: State<'_, AppState>,
    handle: u32,
    path: String,
    chooser: bool,
) -> Result<OpenOutcome, String> {
    let session = state.session(handle)?;
    let archive_path = session.path.clone();
    let dest = session.scratch.clone();

    tauri::async_runtime::spawn_blocking(move || {
        let mut archive = Archive::open(&archive_path).map_err(|e| e.to_string())?;
        let report = archive
            .extract(
                Selection::Paths(std::slice::from_ref(&path)),
                &dest,
                // Flat on purpose: the folders of the archive have no meaning
                // in a scratch directory, and a file two levels down would be
                // harder to find than the one file that was asked for.
                Layout::Flat,
                &mut |_| true,
            )
            .map_err(|e| e.to_string())?;
        if let Some(skipped) = report.skipped.first() {
            return Err(format!("{}: {}", skipped.path, skipped.reason));
        }

        let name = path.rsplit('/').next().unwrap_or(&path);
        let written = dest.join(name);
        let location = written.display().to_string();

        // What the desktop intends to do decides both whether handing the
        // file over is worth trying, and what to call the result: an
        // association pointing at an uninstalled program is skipped rather
        // than followed into nothing, and one that resolves to the system's
        // own picker is reported as a picker and not as an application.
        let association = desktop::association_of(&written);
        if !chooser
            && association != desktop::Association::Dead
            && app
                .opener()
                .open_path(location.clone(), None::<&str>)
                .is_ok()
        {
            return Ok(OpenOutcome {
                path: location,
                action: if association == desktop::Association::Picker {
                    OpenAction::Chooser
                } else {
                    OpenAction::Opened
                },
            });
        }
        if desktop::choose_application(&written) == desktop::Fallback::Chooser {
            return Ok(OpenOutcome {
                path: location,
                action: OpenAction::Chooser,
            });
        }
        app.opener()
            .reveal_item_in_dir(&written)
            .map_err(|e| e.to_string())?;
        Ok(OpenOutcome {
            path: location,
            action: OpenAction::Revealed,
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Extract the whole archive, or the given selection, into `dest_dir`.
#[tauri::command]
async fn extract(
    app: AppHandle,
    state: State<'_, AppState>,
    handle: u32,
    paths: Option<Vec<String>>,
    dest_dir: String,
    layout: Layout,
) -> Result<renzip_core::ExtractionReport, String> {
    let session = state.session(handle)?;
    let archive_path = session.path.clone();
    let cancel = session.cancel.clone();
    cancel.store(false, Ordering::Relaxed);

    tauri::async_runtime::spawn_blocking(move || {
        let mut archive = Archive::open(&archive_path).map_err(|e| e.to_string())?;
        let mut emitter = ProgressEmitter::new(app, "extract", handle);
        let selection = match &paths {
            Some(paths) => Selection::Paths(paths),
            None => Selection::All,
        };
        archive
            .extract(selection, Path::new(&dest_dir), layout, &mut |progress| {
                emitter.emit(progress);
                !cancel.load(Ordering::Relaxed)
            })
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Ask the extraction running on this handle to stop.
#[tauri::command]
fn cancel_extraction(state: State<'_, AppState>, handle: u32) -> Result<(), String> {
    state.session(handle)?.cancel.store(true, Ordering::Relaxed);
    Ok(())
}

/// Pack a directory into a new archive.
#[tauri::command]
async fn build_archive(
    app: AppHandle,
    state: State<'_, AppState>,
    source_dir: String,
    dest_path: String,
    version: String,
) -> Result<renzip_core::BuildReport, String> {
    let version = match version.as_str() {
        "RPA-2.0" => Version::Rpa2_0,
        "RPA-3.0" => Version::Rpa3_0,
        other => return Err(format!("cannot write archives in version {other}")),
    };
    let cancel = state.build_cancel.clone();
    cancel.store(false, Ordering::Relaxed);

    tauri::async_runtime::spawn_blocking(move || {
        let mut emitter = ProgressEmitter::new(app, "build", 0);
        renzip_core::build(
            Path::new(&source_dir),
            Path::new(&dest_path),
            version,
            renzip_core::DEFAULT_KEY,
            &mut |progress| {
                emitter.emit(progress);
                !cancel.load(Ordering::Relaxed)
            },
        )
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Ask the running build to stop.
#[tauri::command]
fn cancel_build(state: State<'_, AppState>) {
    state.build_cancel.store(true, Ordering::Relaxed);
}

/// Sends progress events to the window, no faster than the window can read.
struct ProgressEmitter {
    app: AppHandle,
    phase: &'static str,
    handle: u32,
    last: Instant,
}

impl ProgressEmitter {
    /// Start emitting for one operation.
    fn new(app: AppHandle, phase: &'static str, handle: u32) -> Self {
        ProgressEmitter {
            app,
            phase,
            handle,
            // Backdated so the first entry of a run is always reported.
            last: Instant::now() - PROGRESS_INTERVAL,
        }
    }

    /// Emit this progress update unless one was just sent.
    fn emit(&mut self, progress: &Progress<'_>) {
        let is_last = progress.done == progress.total;
        if !is_last && self.last.elapsed() < PROGRESS_INTERVAL {
            return;
        }
        self.last = Instant::now();
        let _ = self.app.emit(
            PROGRESS_EVENT,
            ProgressEvent {
                phase: self.phase,
                handle: self.handle,
                done: progress.done,
                total: progress.total,
                bytes_done: progress.bytes_done,
                bytes_total: progress.bytes_total,
                current: progress.current.to_string(),
            },
        );
    }
}

/// File name of a path, or the whole path when it has none.
fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string())
}

/// Build and run the application.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            app.manage(AppState::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_archive,
            list_entries,
            close_archive,
            entry_association,
            scan_directory,
            open_entry,
            preview_entry,
            extract,
            cancel_extraction,
            build_archive,
            cancel_build,
        ])
        .build(tauri::generate_context!())
        .expect("error while starting RenZip")
        // The scratch copies the row menu made are cleaned up on the way out,
        // the same as when an archive is closed. A crash still leaves them
        // behind, which is what the system temporary folder is for.
        .run(|app, event| {
            if matches!(event, tauri::RunEvent::Exit) {
                app.state::<AppState>().close_all();
            }
        });
}
