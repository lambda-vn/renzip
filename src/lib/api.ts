/**
 * Typed wrappers over the Tauri commands exposed by src-tauri.
 *
 * Every shape here mirrors a Rust type; keeping them in one file is what
 * makes a change on the Rust side show up as a type error rather than as a
 * silent undefined at runtime.
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/** An entry the archive announced but that will not be extracted. */
export type Rejection = {
  path: string;
  reason: string;
};

/** What is known about an archive right after it is opened. */
export type ArchiveSummary = {
  handle: number;
  name: string;
  path: string;
  version: string;
  entry_count: number;
  total_size: number;
  file_size: number;
  rejected: Rejection[];
  multi_segment: string[];
};

/** One entry of an open archive. */
export type EntryRow = {
  path: string;
  size: number;
  offset: number;
  prefix_len: number;
};

/** One file a build would pack, as found on disk. */
export type SourceFile = {
  path: string;
  size: number;
};

/** What a walk of a source directory found. */
export type ScanReport = {
  files: SourceFile[];
  skipped: Rejection[];
};

/** How extracted files are laid out under the destination directory. */
export type Layout = "Tree" | "Flat";

/** What the window should render for one entry. */
export type Preview =
  | { kind: "text"; content: string }
  | { kind: "image"; data_url: string }
  | { kind: "binary"; size: number; reason: string };

/** Outcome of an extraction. */
export type ExtractionReport = {
  extracted: number;
  bytes: number;
  skipped: Rejection[];
  cancelled: boolean;
};

/** Outcome of a build. */
export type BuildReport = {
  entries: number;
  bytes: number;
  skipped: Rejection[];
  cancelled: boolean;
};

/** Progress of a running extraction or build. */
export type ProgressPayload = {
  phase: "extract" | "build";
  handle: number;
  done: number;
  total: number;
  bytes_done: number;
  bytes_total: number;
  current: string;
};

/** Open an archive and get its summary. */
export function openArchive(path: string): Promise<ArchiveSummary> {
  return invoke<ArchiveSummary>("open_archive", { path });
}

/** List every usable entry of an open archive. */
export function listEntries(handle: number): Promise<EntryRow[]> {
  return invoke<EntryRow[]>("list_entries", { handle });
}

/** Forget an archive. */
export function closeArchive(handle: number): Promise<void> {
  return invoke<void>("close_archive", { handle });
}

/** Read one entry and decide how to show it. */
export function previewEntry(handle: number, path: string): Promise<Preview> {
  return invoke<Preview>("preview_entry", { handle, path });
}

/**
 * Extract an archive into a directory.
 *
 * @param paths - Entries to extract, or null for the whole archive.
 * @param layout - Keep the archive folders, or write every file side by side.
 */
export function extract(
  handle: number,
  paths: string[] | null,
  destDir: string,
  layout: Layout,
): Promise<ExtractionReport> {
  return invoke<ExtractionReport>("extract", { handle, paths, destDir, layout });
}

/** What opening an entry ended up doing. */
export type OpenOutcome = {
  path: string;
  /** "opened" by an application, "chooser" shown, or file "revealed". */
  action: "opened" | "chooser" | "revealed";
};

/**
 * Write one entry to a temporary directory and hand it to the desktop.
 *
 * The backend picks the path and does the opening, so the window never names
 * a file for the system to run.
 *
 * @param chooser - Skip the registered application and ask the desktop for
 *   its "open with" picker straight away.
 */
export function openEntry(
  handle: number,
  path: string,
  chooser = false,
): Promise<OpenOutcome> {
  return invoke<OpenOutcome>("open_entry", { handle, path, chooser });
}

/** What the desktop would do with a file carrying a given extension. */
export type Association = "runnable" | "picker" | "dead" | "unknown";

/**
 * Ask what the desktop would open this kind of file with.
 *
 * @param extension - Extension without its dot.
 */
export function entryAssociation(extension: string): Promise<Association> {
  return invoke<Association>("entry_association", { extension });
}

/** List what packing a directory would put into an archive. */
export function scanDirectory(sourceDir: string): Promise<ScanReport> {
  return invoke<ScanReport>("scan_directory", { sourceDir });
}

/** Ask the running extraction to stop. */
export function cancelExtraction(handle: number): Promise<void> {
  return invoke<void>("cancel_extraction", { handle });
}

/**
 * Pack a directory into a new archive.
 *
 * @param excludes - Files or folders of `sourceDir` to leave out, at any
 *   depth, each an archive path with no trailing separator on a folder.
 */
export function buildArchive(
  sourceDir: string,
  destPath: string,
  version: string,
  excludes: string[],
): Promise<BuildReport> {
  return invoke<BuildReport>("build_archive", { sourceDir, destPath, version, excludes });
}

/** Ask the running build to stop. */
export function cancelBuild(): Promise<void> {
  return invoke<void>("cancel_build");
}

/** The .rpa path the desktop launched RenZip with, if there was one. */
export function launchPath(): Promise<string | null> {
  return invoke<string | null>("launch_path");
}

/** Subscribe to progress events until the returned function is called. */
export function onProgress(
  handler: (payload: ProgressPayload) => void,
): Promise<UnlistenFn> {
  return listen<ProgressPayload>("renzip://progress", (event) =>
    handler(event.payload),
  );
}
