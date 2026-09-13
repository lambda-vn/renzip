/**
 * Typed wrappers over the Tauri commands exposed by src-tauri.
 *
 * Every shape here mirrors a Rust type; keeping them in one file is what
 * makes a change on the Rust side show up as a type error rather than as a
 * silent undefined at runtime.
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/** Why an entry path cannot be trusted, mirroring `PathRejection`. */
export type PathRejection =
  | { kind: "empty" | "absolute" | "drive_letter" | "traversal" }
  | { kind: "reserved_name" | "forbidden_character" | "trailing_dot_or_space"; detail: string };

/** What a failed file system operation was doing, mirroring `IoAction`. */
export type IoAction =
  | "create"
  | "resolve"
  | "replace_link"
  | "write"
  | "read"
  | "extract"
  | "seek"
  | "store";

/** Why a file was left out or refused, mirroring `Reason`: a code and its details. */
export type Reason =
  | {
      code:
        | "unreadable_segments"
        | "no_file_name"
        | "no_parent_directory"
        | "destination_of_build"
        | "symbolic_link"
        | "archive_in_archive";
    }
  | { code: "unsafe_path" | "unusable_name"; rejection: PathRejection }
  | { code: "prefix_too_long"; prefix: number; length: number }
  | { code: "past_end" | "segment_past_end"; start: number; end: number; size: number }
  | { code: "range_overflow" | "segment_overflow"; offset: number; length: number }
  | { code: "duplicate_path" | "already_written"; path: string }
  | { code: "outside_destination"; root: string }
  | { code: "archive_ends_early"; written: number; announced: number }
  | { code: "io"; action: IoAction; path: string; error: string };

/** An entry the archive announced but that will not be extracted. */
export type Rejection = {
  path: string;
  reason: Reason;
};

/** What is wrong with an archive index, mirroring `IndexProblem`. */
export type IndexProblem =
  | { kind: "not_a_dict" }
  | { kind: "inflate" | "unpickle" | "pickle" | "deflate"; error: string }
  | { kind: "too_large"; limit: number }
  | { kind: "key_not_a_string"; key: string }
  | { kind: "not_a_list" | "no_tuple" | "not_a_tuple"; entry: string }
  | { kind: "field_count"; entry: string; count: number }
  | { kind: "prefix_not_bytes" | "not_a_positive_integer"; entry: string; value: string };

/** Where an archive ends before what it announces, mirroring `Truncation`. */
export type Truncation =
  | { kind: "index_past_end"; offset: number; size: number }
  | { kind: "entry_stops_early"; entry: string; read: number; length: number };

/**
 * Why a command failed, mirroring `CommandError` and the core `Error` it
 * passes through: a code and its details, which the window words itself.
 */
export type CommandError =
  | { code: "io"; path: string; error: string }
  | { code: "unsupported_header"; header: string }
  | { code: "missing_index_file"; path: string }
  | { code: "corrupt_index"; problem: IndexProblem }
  | { code: "truncated"; truncation: Truncation }
  | { code: "entry_not_found"; entry: string }
  | { code: "invalid_destination" | "invalid_source"; path: string; error: string | null }
  | { code: "unsupported_write_version"; version: string }
  | { code: "archive_not_open"; handle: number }
  | { code: "entry_skipped"; entry: string; reason: Reason }
  | { code: "reveal_failed" | "internal"; error: string };

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

/** Why an entry has nothing to show, mirroring `BinaryReason`. */
export type BinaryReason = { code: "too_large"; limit: number } | { code: "not_text" };

/** What the window should render for one entry. */
export type Preview =
  | { kind: "text"; content: string }
  | { kind: "image"; data_url: string }
  | { kind: "binary"; size: number; reason: BinaryReason };

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
