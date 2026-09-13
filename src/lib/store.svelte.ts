/**
 * Everything the window knows, in one place.
 *
 * The screens share the same archive, the same selection and the same running
 * operation, so they read one store rather than passing that state down
 * through props.
 */

import { message } from "@tauri-apps/plugin-dialog";
import { SvelteSet } from "svelte/reactivity";

import {
  buildArchive,
  cancelBuild,
  cancelExtraction,
  closeArchive,
  extract,
  listEntries,
  openArchive,
  previewEntry,
  scanDirectory,
  type ArchiveSummary,
  type EntryRow,
  type Layout,
  type Preview,
  type ProgressPayload,
  type Rejection,
  type ScanReport,
} from "./api";
import { describeError, formatSize } from "./format";
import { setLanguage, t, tn } from "./i18n.svelte";
import { kindKey, kindOf } from "./kinds";
import { applyTheme, loadSettings, saveSettings, type Settings } from "./settings";
import { buildSourceTree, visibleSourceRows } from "./sourceTree";
import { buildTree, descendants, parentOf, visibleRows, type TreeRow } from "./tree";

/** Which screen is showing. */
export type View = "browse" | "create" | "settings";

/** A line shown in the footer. */
export type Status = { tone: "info" | "warn"; text: string };

/** The single store the views read and write. */
class AppStore {
  /** Screen currently shown. */
  view = $state<View>("browse");
  /** Open archive, null before anything is opened. */
  archive = $state<ArchiveSummary | null>(null);
  /** Every entry of that archive. */
  entries = $state<EntryRow[]>([]);
  /** Folders unfolded in the list. */
  open = new SvelteSet<string>();
  /** Paths of the entries picked for extraction. */
  selected = new SvelteSet<string>();
  /** Text typed in the search box. */
  filter = $state("");

  /** Entry shown in the selection panel. */
  current = $state<EntryRow | null>(null);
  /** What the backend decided to show for it. */
  preview = $state<Preview | null>(null);
  /** True while that entry is being read. */
  previewLoading = $state(false);

  /** Progress of the running extraction or build. */
  progress = $state<ProgressPayload | null>(null);
  /** True while an operation runs, which disables the toolbar. */
  busy = $state(false);
  /** Last thing worth telling the user. */
  status = $state<Status | null>(null);
  /** Entries or files left out by the last operation. */
  skipped = $state<Rejection[]>([]);

  /** Directory picked on the creation screen, with what it holds. */
  source = $state<{ dir: string; report: ScanReport } | null>(null);
  /** Files or folders of the staged directory left out of the build. */
  excluded = new SvelteSet<string>();
  /** Folders of the staged directory unfolded in the creation screen. */
  stagedOpen = new SvelteSet<string>();
  /** Where the archive being created will be written. */
  destPath = $state("");
  /** Version the archive being created will announce. */
  version = $state<string>("RPA-3.0");
  /** Where an extraction will write. */
  destDir = $state("");
  /** Whether an extraction keeps the archive folders. */
  layout = $state<Layout>("Tree");

  /** Persisted preferences. */
  settings = $state<Settings>(loadSettings());

  /** Folder tree of the open archive. */
  tree = $derived(buildTree(this.entries));
  /** Rows of the single list: folders to unfold, and files. */
  rows = $derived(this.listing());
  /** How many files of each folder are ticked, for the tri-state boxes. */
  tally = $derived(this.countByFolder());
  /** Entries by path, so the selection can be read without a scan. */
  byPath = $derived(new Map(this.entries.map((entry) => [entry.path, entry])));
  /** How many entries are picked. */
  selectionSize = $derived(this.selected.size);
  /** What the selection holds, for the panel that stands in for a preview. */
  selection = $derived(this.summarize());

  /** Staged files still due for the build, once dropped entries are out. */
  stagedFiles = $derived(
    (this.source?.report.files ?? []).filter((file) => !this.isExcluded(file.path)),
  );
  /** Folder tree of what is still staged. */
  stagedTree = $derived(buildSourceTree(this.stagedFiles));
  /** Rows of the staged content list: folders to unfold, and files. */
  stagedRows = $derived(visibleSourceRows(this.stagedTree, this.stagedOpen));

  /** Apply the stored preferences as soon as the window has a document. */
  init(): void {
    applyTheme(this.settings.theme);
    setLanguage(this.settings.language);
    this.version = this.settings.defaultVersion;
  }

  /**
   * Add up the selection: how many, how big, and of what.
   *
   * Reads the selection rather than every entry, so picking a thousand files
   * costs a thousand lookups and not a scan of the whole archive.
   */
  private summarize(): {
    count: number;
    size: number;
    kinds: { label: string; count: number }[];
    names: string[];
  } {
    const tally = new Map<string, number>();
    let size = 0;
    const names: string[] = [];

    for (const path of this.selected) {
      const entry = this.byPath.get(path);
      if (!entry) {
        continue;
      }
      size += entry.size;
      const label = t(kindKey(kindOf(path)));
      tally.set(label, (tally.get(label) ?? 0) + 1);
      if (names.length < 6) {
        names.push(path);
      }
    }

    return {
      count: this.selected.size,
      size,
      kinds: [...tally.entries()]
        .map(([label, count]) => ({ label, count }))
        .sort((a, b) => b.count - a.count),
      names,
    };
  }

  /** Rows to show, once the search is applied. */
  private listing(): TreeRow[] {
    const filter = this.filter.trim().toLowerCase();
    const keep = (entry: EntryRow): boolean =>
      filter === "" || entry.path.toLowerCase().includes(filter);
    return visibleRows(this.tree, this.open, keep, filter !== "");
  }

  /**
   * Count the ticked files of every folder, the root included.
   *
   * Walks the selection and climbs each path's ancestors, rather than
   * scanning the archive per folder row: a folder row then reads its state in
   * constant time, whatever its subtree holds.
   */
  private countByFolder(): Map<string, number> {
    const tally = new Map<string, number>();
    for (const path of this.selected) {
      let folder = parentOf(path);
      for (;;) {
        tally.set(folder, (tally.get(folder) ?? 0) + 1);
        if (folder === "") {
          break;
        }
        folder = parentOf(folder);
      }
    }
    return tally;
  }

  /**
   * Open an archive and show it.
   *
   * @param path - Full path of the .rpa file.
   */
  async load(path: string): Promise<void> {
    try {
      if (this.archive) {
        await closeArchive(this.archive.handle);
      }
      const summary = await openArchive(path);
      this.entries = await listEntries(summary.handle);
      this.archive = summary;
      this.view = "browse";
      this.filter = "";
      this.open.clear();
      this.selected.clear();
      this.current = null;
      this.preview = null;
      this.skipped = summary.rejected;
      this.status =
        summary.rejected.length > 0
          ? { tone: "warn", text: tn("count.rejected", summary.rejected.length) }
          : { tone: "info", text: t("status.archiveOpened", { name: summary.name }) };
    } catch (error) {
      this.archive = null;
      this.entries = [];
      this.skipped = [];
      this.status = { tone: "warn", text: describeError(error) };
    }
  }

  /**
   * Re-read the open archive from disk, keeping what the window shows.
   *
   * Unlike load(), the search, the folders left open and the selection all
   * survive: F5 is meant to catch up with a file that changed on disk, not
   * to put the window back to how it looked when the archive was first
   * opened. An entry a refresh no longer finds is simply dropped from the
   * selection, and from the panel if it was the one shown.
   */
  async refresh(): Promise<void> {
    if (!this.archive || this.busy) {
      return;
    }
    const path = this.archive.path;
    const currentPath = this.current?.path;
    try {
      await closeArchive(this.archive.handle);
      const summary = await openArchive(path);
      const entries = await listEntries(summary.handle);
      const byPath = new Map(entries.map((entry) => [entry.path, entry]));

      for (const selected of [...this.selected]) {
        if (!byPath.has(selected)) {
          this.selected.delete(selected);
        }
      }

      this.archive = summary;
      this.entries = entries;
      this.skipped = summary.rejected;
      this.status =
        summary.rejected.length > 0
          ? { tone: "warn", text: tn("count.rejected", summary.rejected.length) }
          : { tone: "info", text: t("status.refreshed") };

      const stillThere = currentPath ? byPath.get(currentPath) : undefined;
      if (stillThere) {
        await this.show(stillThere, true);
      } else {
        this.current = null;
        this.preview = null;
      }
    } catch (error) {
      this.archive = null;
      this.entries = [];
      this.current = null;
      this.preview = null;
      this.selected.clear();
      this.skipped = [];
      this.status = { tone: "warn", text: describeError(error) };
    }
  }

  /** Unfold or fold a folder of the list. */
  toggleFolder(key: string): void {
    if (this.open.has(key)) {
      this.open.delete(key);
    } else {
      this.open.add(key);
    }
  }

  /** Tick or untick a row, whether it is a file or a whole folder. */
  toggleRow(row: TreeRow): void {
    if (row.kind === "file") {
      this.toggle(row.key);
    } else {
      this.toggleFolderSelection(row.key);
    }
  }

  /** Add or remove one entry from the selection. */
  toggle(path: string): void {
    if (this.selected.has(path)) {
      this.selected.delete(path);
    } else {
      this.selected.add(path);
    }
  }

  /** Put every file of a folder in the selection, or take them all out. */
  toggleFolderSelection(key: string): void {
    const paths = descendants(this.tree, key).map((entry) => entry.path);
    const allIn = paths.length > 0 && paths.every((path) => this.selected.has(path));
    for (const path of paths) {
      if (allIn) {
        this.selected.delete(path);
      } else {
        this.selected.add(path);
      }
    }
  }

  /** Put every entry of the archive in the selection. */
  selectAll(): void {
    for (const entry of this.entries) {
      this.selected.add(entry.path);
    }
  }

  /** Unfold or fold a folder of the staged content list. */
  toggleStagedFolder(key: string): void {
    if (this.stagedOpen.has(key)) {
      this.stagedOpen.delete(key);
    } else {
      this.stagedOpen.add(key);
    }
  }

  /**
   * Whether a staged path was dropped from the build.
   *
   * A path is out once it, or a folder above it, was excluded: dropping
   * "images" must also drop "images/bg.png" without every file needing its
   * own entry in the set.
   *
   * @param path - Staged file path to test.
   */
  isExcluded(path: string): boolean {
    for (const excluded of this.excluded) {
      if (path === excluded || path.startsWith(`${excluded}/`)) {
        return true;
      }
    }
    return false;
  }

  /**
   * Read one entry and show it in the selection panel.
   *
   * @param entry - Entry to inspect.
   * @param force - Read it even when automatic preview is off.
   */
  async show(entry: EntryRow, force = false): Promise<void> {
    this.current = entry;
    this.preview = null;
    if (!this.archive || (!this.settings.autoPreview && !force)) {
      return;
    }
    this.previewLoading = true;
    try {
      this.preview = await previewEntry(this.archive.handle, entry.path);
    } catch (error) {
      this.status = { tone: "warn", text: describeError(error) };
    } finally {
      this.previewLoading = false;
    }
  }

  /**
   * Extract entries into the destination of the extraction screen.
   *
   * @param paths - Entries to extract, or null for the whole archive.
   */
  async runExtraction(paths: string[] | null): Promise<void> {
    if (!this.archive || this.destDir === "") {
      this.status = { tone: "warn", text: t("panel.destination.pick") };
      return;
    }

    this.busy = true;
    this.skipped = [];
    this.progress = null;
    try {
      const report = await extract(this.archive.handle, paths, this.destDir, this.layout);
      this.skipped = report.skipped;
      this.status = {
        tone: report.skipped.length > 0 || report.cancelled ? "warn" : "info",
        text:
          `${tn("count.extracted", report.extracted)} (${formatSize(report.bytes)})` +
          `${report.cancelled ? t("status.cancelled") : ""}` +
          `${report.skipped.length > 0 ? `, ${tn("count.ignored", report.skipped.length)}` : ""}.`,
      };
    } catch (error) {
      this.status = { tone: "warn", text: describeError(error) };
    } finally {
      this.busy = false;
      this.progress = null;
    }
  }

  /**
   * Read a directory and stage it on the creation screen.
   *
   * @param dir - Directory to pack.
   */
  async stage(dir: string): Promise<void> {
    try {
      const report = await scanDirectory(dir);
      this.source = { dir, report };
      this.excluded.clear();
      this.stagedOpen.clear();
      this.skipped = report.skipped;
      this.status = {
        tone: "info",
        text: `${tn("count.packed", report.files.length)} (${formatSize(
          report.files.reduce((total, file) => total + file.size, 0),
        )}).`,
      };
      await warnAboutSkippedArchives(report.skipped);
    } catch (error) {
      this.source = null;
      this.status = { tone: "warn", text: describeError(error) };
    }
  }

  /**
   * Drop a file or folder of the staged directory from the next build.
   *
   * @param key - Archive name of a file, or path of a folder, as shown on
   *   the creation screen. Dropping a folder drops everything under it.
   */
  excludeFromSource(key: string): void {
    this.excluded.add(key);
  }

  /** Pack the staged directory, then open the result to check it. */
  async runBuild(): Promise<void> {
    if (!this.source || this.destPath === "") {
      this.status = { tone: "warn", text: t("create.nothing") };
      return;
    }

    this.busy = true;
    this.skipped = [];
    this.progress = null;
    try {
      const report = await buildArchive(
        this.source.dir,
        this.destPath,
        this.version,
        [...this.excluded],
      );
      this.skipped = report.skipped;

      if (report.cancelled) {
        this.status = {
          tone: "warn",
          text: t("status.buildCancelled", { path: this.destPath }),
        };
        return;
      }

      let text = t("status.built", {
        path: this.destPath,
        count: tn("count.packed", report.entries),
        size: formatSize(report.bytes),
        version: this.version,
      });

      if (this.settings.verifyAfterBuild) {
        // Verification is a real read of what was just written: the archive is
        // reopened, its index decoded, and what it announces compared to what
        // was packed. Opening it also puts it in front of the user, which is
        // where they wanted to end up anyway.
        const before = { entries: report.entries, bytes: report.bytes };
        await this.load(this.destPath);
        const summary = this.archive;
        const ok =
          summary !== null &&
          summary.entry_count === before.entries &&
          summary.total_size === before.bytes;
        text += ok
          ? t("status.verified")
          : t("status.verifyFailed", {
              reason: t("status.verifyMismatch", {
                found: tn("count.entry", summary?.entry_count ?? 0),
                expected: tn("count.packed", before.entries),
              }),
            });
        this.status = { tone: ok ? "info" : "warn", text };
        return;
      }

      this.status = { tone: "info", text };
    } catch (error) {
      this.status = { tone: "warn", text: describeError(error) };
    } finally {
      this.busy = false;
      this.progress = null;
    }
  }

  /** Ask whatever is running to stop. */
  async cancel(): Promise<void> {
    if (this.progress?.phase === "build") {
      await cancelBuild();
    } else if (this.archive) {
      await cancelExtraction(this.archive.handle);
    }
  }

  /**
   * Change one preference and persist the lot.
   *
   * @param patch - Fields to change.
   */
  updateSettings(patch: Partial<Settings>): void {
    this.settings = { ...this.settings, ...patch };
    saveSettings(this.settings);
    if (patch.language) {
      setLanguage(patch.language);
    }
    if (patch.defaultVersion) {
      this.version = patch.defaultVersion;
    }
  }
}

/**
 * Explain, once, that scan already left every .rpa/.rpi out of what was just
 * staged.
 *
 * Ren'Py never opens an archive packed inside another, so these were never
 * going into the build regardless; without this, that only shows up as an
 * unlabeled gap between what a folder holds and what got staged, the first
 * time someone stages one with an old archive still sitting in it.
 *
 * @param skipped - Everything scan left out, most of it for unrelated
 *   reasons (a symbolic link, an unusable name).
 */
async function warnAboutSkippedArchives(skipped: Rejection[]): Promise<void> {
  const archives = skipped.filter((item) => /\.(rpa|rpi)$/i.test(item.path));
  if (archives.length === 0) {
    return;
  }
  const shown = archives.slice(0, 5);
  const remaining = archives.length - shown.length;
  const list =
    shown.map((item) => `- ${item.path}`).join("\n") +
    (remaining > 0 ? `\n${tn("count.other", remaining)}` : "");
  await message(
    t("create.archivesSkipped.body", { count: tn("count.file", archives.length), list }),
    { title: t("create.archivesSkipped.title"), kind: "info" },
  );
}

/** The store, shared by every view. */
export const app = new AppStore();
