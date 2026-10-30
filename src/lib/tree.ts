/**
 * Virtual folder tree over a flat list of paths.
 *
 * An .rpa has no directories, only paths with slashes in them. The tree is
 * built once per archive, in one pass, because rebuilding it on every folder
 * change would be felt on the fifty thousand entry archives this application
 * is meant to survive. The files a build would pack are shown the same way,
 * so the tree takes anything with a path and a size: an archive's entries,
 * or a staged directory's files.
 */

import type { EntryRow } from "./api";

/** What a tree is built from: a slash separated path and a size. */
type Leaf = { path: string; size: number };

/** Content of one virtual folder. */
type Folder<T extends Leaf> = {
  dirs: Set<string>;
  files: T[];
  /** Recursive totals, for the size shown on a folder row. */
  size: number;
  count: number;
};

/** Every folder, keyed by path ("" is the root). */
export type Tree<T extends Leaf = EntryRow> = Map<string, Folder<T>>;

/** One row of the single list: a folder to unfold, or a file. */
export type TreeRow<T extends Leaf = EntryRow> =
  | {
      kind: "dir";
      /** Folder path, which is also the row identity. */
      key: string;
      /** Name as displayed, with its trailing slash. */
      name: string;
      /** Nesting level, 0 at the root. */
      depth: number;
      /** Files it holds, recursively. */
      count: number;
      /** Total size of those files. */
      size: number;
      /** True when it holds anything at all. */
      hasKids: boolean;
      /** True when its content is currently shown. */
      open: boolean;
    }
  | {
      kind: "file";
      /** File path, which is also the row identity. */
      key: string;
      /** Name as displayed: the file name, or the whole path when filtering. */
      name: string;
      /** Nesting level, 0 at the root. */
      depth: number;
      /** The entry or file itself. */
      entry: T;
    };

/** Build the folder tree of an entry or file list. */
export function buildTree<T extends Leaf>(entries: T[]): Tree<T> {
  const tree: Tree<T> = new Map();
  folderOf(tree, "");

  for (const entry of entries) {
    const segments = entry.path.split("/");
    segments.pop();

    let parent = "";
    for (const segment of segments) {
      const current = parent === "" ? segment : `${parent}/${segment}`;
      folderOf(tree, parent).dirs.add(segment);
      folderOf(tree, current);
      parent = current;
    }
    folderOf(tree, parent).files.push(entry);

    // Totals climb back up every ancestor, the root included.
    let ancestor: string | null = parent;
    while (ancestor !== null) {
      const folder = folderOf(tree, ancestor);
      folder.size += entry.size;
      folder.count += 1;
      ancestor = ancestor === "" ? null : parentOf(ancestor);
    }
  }

  for (const folder of tree.values()) {
    folder.files.sort((a, b) => a.path.localeCompare(b.path));
  }
  return tree;
}

/**
 * Flatten the tree into the rows the single list shows.
 *
 * Folders come before files at every level, and only what sits under an open
 * folder is produced, so an archive of fifty thousand entries costs one pass
 * over what is actually visible.
 *
 * A filter short-circuits the whole thing: matching files are listed flat
 * with their full path, because unfolding a tree to reveal three matches
 * buried at four levels helps nobody.
 *
 * @param tree - Tree of the open archive or the staged directory.
 * @param open - Paths of the folders currently unfolded.
 * @param keep - Optional test a file must pass; null lists everything.
 */
export function visibleRows<T extends Leaf>(
  tree: Tree<T>,
  open: Set<string>,
  keep: ((entry: T) => boolean) | null = null,
  filtering = false,
): TreeRow<T>[] {
  if (filtering) {
    const matches: TreeRow<T>[] = [];
    for (const folder of tree.values()) {
      for (const entry of folder.files) {
        if (!keep || keep(entry)) {
          matches.push({ kind: "file", key: entry.path, name: entry.path, depth: 0, entry });
        }
      }
    }
    return matches.sort((a, b) => a.key.localeCompare(b.key));
  }

  const rows: TreeRow<T>[] = [];
  const walk = (path: string, depth: number): void => {
    const folder = tree.get(path);
    if (!folder) {
      return;
    }
    for (const name of [...folder.dirs].sort((a, b) => a.localeCompare(b))) {
      const key = path === "" ? name : `${path}/${name}`;
      const child = tree.get(key);
      const isOpen = open.has(key);
      rows.push({
        kind: "dir",
        key,
        name: `${name}/`,
        depth,
        count: child?.count ?? 0,
        size: child?.size ?? 0,
        hasKids: (child?.dirs.size ?? 0) + (child?.files.length ?? 0) > 0,
        open: isOpen,
      });
      if (isOpen) {
        walk(key, depth + 1);
      }
    }
    for (const entry of folder.files) {
      if (!keep || keep(entry)) {
        rows.push({
          kind: "file",
          key: entry.path,
          name: baseName(entry.path),
          depth,
          entry,
        });
      }
    }
  };
  walk("", 0);
  return rows;
}

/** Every file below a folder, that folder's own files included. */
export function descendants<T extends Leaf>(tree: Tree<T>, path: string): T[] {
  const folder = tree.get(path);
  if (!folder) {
    return [];
  }
  const files = [...folder.files];
  for (const name of folder.dirs) {
    files.push(...descendants(tree, path === "" ? name : `${path}/${name}`));
  }
  return files.sort((a, b) => a.path.localeCompare(b.path));
}

/** Parent of a folder path, "" for a first level folder. */
export function parentOf(path: string): string {
  const cut = path.lastIndexOf("/");
  return cut === -1 ? "" : path.slice(0, cut);
}

/** Name of a file, without its folders. */
export function baseName(path: string): string {
  return path.slice(path.lastIndexOf("/") + 1);
}

/** Get a folder, creating it the first time it is named. */
function folderOf<T extends Leaf>(tree: Tree<T>, path: string): Folder<T> {
  let folder = tree.get(path);
  if (!folder) {
    folder = { dirs: new Set(), files: [], size: 0, count: 0 };
    tree.set(path, folder);
  }
  return folder;
}

/** Rows a page key moves by: about one screen of the list. */
const PAGE = 12;

/**
 * Apply a navigation key of the tree view pattern to a list of rows.
 *
 * The arrows walk, Home and End jump to either end, the page keys move by
 * a screen; Right unfolds a closed folder or steps into an open one, Left
 * folds an open one. What a key does beyond moving (ticking, opening,
 * excluding) belongs to each tree, which handles it when this answers
 * undefined.
 *
 * @param key - The key, as KeyboardEvent.key names it.
 * @param rows - Rows on screen.
 * @param active - Index of the active row.
 * @param toggle - Folds or unfolds the folder with this key.
 * @returns The row to make active, null when the key only folded or
 *   unfolded, undefined when it is not a navigation key.
 */
export function navigate<T extends Leaf>(
  key: string,
  rows: TreeRow<T>[],
  active: number,
  toggle: (key: string) => void,
): number | null | undefined {
  const row = rows[active];
  const last = rows.length - 1;
  switch (key) {
    case "ArrowDown":
      return Math.min(last, active + 1);
    case "ArrowUp":
      return Math.max(0, active - 1);
    case "Home":
      return 0;
    case "End":
      return last;
    case "PageDown":
      return Math.min(last, active + PAGE);
    case "PageUp":
      return Math.max(0, active - PAGE);
    case "ArrowRight":
      if (row.kind === "dir" && !row.open && row.hasKids) {
        toggle(row.key);
      } else if (row.kind === "dir" && row.open) {
        return Math.min(last, active + 1);
      }
      return null;
    case "ArrowLeft":
      if (row.kind === "dir" && row.open) {
        toggle(row.key);
      }
      return null;
    default:
      return undefined;
  }
}
