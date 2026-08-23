/**
 * Virtual folder tree over the flat entry list of an archive.
 *
 * An .rpa has no directories, only paths with slashes in them. The tree is
 * built once per archive, in one pass, because rebuilding it on every folder
 * change would be felt on the fifty thousand entry archives this application
 * is meant to survive.
 */

import type { EntryRow } from "./api";

/** Content of one virtual folder. */
type Folder = {
  dirs: Set<string>;
  files: EntryRow[];
  /** Recursive totals, for the size shown on a folder row. */
  size: number;
  count: number;
};

/** Every folder of an archive, keyed by path ("" is the root). */
export type Tree = Map<string, Folder>;

/** One row of the single list: a folder to unfold, or a file. */
export type TreeRow =
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
      /** Entry path, which is also the row identity. */
      key: string;
      /** Name as displayed: the file name, or the whole path when filtering. */
      name: string;
      /** Nesting level, 0 at the root. */
      depth: number;
      /** The entry itself. */
      entry: EntryRow;
    };

/** Build the folder tree of an entry list. */
export function buildTree(entries: EntryRow[]): Tree {
  const tree: Tree = new Map();
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
 * @param tree - Tree of the open archive.
 * @param open - Paths of the folders currently unfolded.
 * @param keep - Optional test a file must pass; null lists everything.
 */
export function visibleRows(
  tree: Tree,
  open: Set<string>,
  keep: ((entry: EntryRow) => boolean) | null = null,
  filtering = false,
): TreeRow[] {
  if (filtering) {
    const matches: TreeRow[] = [];
    for (const folder of tree.values()) {
      for (const entry of folder.files) {
        if (!keep || keep(entry)) {
          matches.push({ kind: "file", key: entry.path, name: entry.path, depth: 0, entry });
        }
      }
    }
    return matches.sort((a, b) => a.key.localeCompare(b.key));
  }

  const rows: TreeRow[] = [];
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
export function descendants(tree: Tree, path: string): EntryRow[] {
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
function folderOf(tree: Tree, path: string): Folder {
  let folder = tree.get(path);
  if (!folder) {
    folder = { dirs: new Set(), files: [], size: 0, count: 0 };
    tree.set(path, folder);
  }
  return folder;
}
