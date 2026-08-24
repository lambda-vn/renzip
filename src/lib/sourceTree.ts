/**
 * Virtual folder tree over the files a build would pack.
 *
 * Mirrors tree.ts's shape for an open archive, but built straight from the
 * plain SourceFile list a directory scan reports: the creation screen has no
 * offsets or extraction state to carry per row, only a path and a size.
 */

import type { SourceFile } from "./api";
import { baseName, parentOf } from "./tree";

/** Content of one virtual folder. */
type Folder = {
  dirs: Set<string>;
  files: SourceFile[];
  /** Recursive totals, for the size shown on a folder row. */
  size: number;
  count: number;
};

/** Every folder of a staged directory, keyed by path ("" is the root). */
export type SourceTree = Map<string, Folder>;

/** One row of the staged content list: a folder to unfold, or a file. */
export type SourceRow =
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
      /** Name as displayed: the file name. */
      name: string;
      /** Nesting level, 0 at the root. */
      depth: number;
      /** Size of the file. */
      size: number;
    };

/** Build the folder tree of a staged file list. */
export function buildSourceTree(files: SourceFile[]): SourceTree {
  const tree: SourceTree = new Map();
  folderOf(tree, "");

  for (const file of files) {
    const segments = file.path.split("/");
    segments.pop();

    let parent = "";
    for (const segment of segments) {
      const current = parent === "" ? segment : `${parent}/${segment}`;
      folderOf(tree, parent).dirs.add(segment);
      folderOf(tree, current);
      parent = current;
    }
    folderOf(tree, parent).files.push(file);

    // Totals climb back up every ancestor, the root included.
    let ancestor: string | null = parent;
    while (ancestor !== null) {
      const folder = folderOf(tree, ancestor);
      folder.size += file.size;
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
 * Flatten the tree into the rows the staged content list shows.
 *
 * Folders come before files at every level, and only what sits under an open
 * folder is produced.
 *
 * @param tree - Tree of the staged directory.
 * @param open - Paths of the folders currently unfolded.
 */
export function visibleSourceRows(tree: SourceTree, open: Set<string>): SourceRow[] {
  const rows: SourceRow[] = [];
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
    for (const file of folder.files) {
      rows.push({ kind: "file", key: file.path, name: baseName(file.path), depth, size: file.size });
    }
  };
  walk("", 0);
  return rows;
}

/** Get a folder, creating it the first time it is named. */
function folderOf(tree: SourceTree, path: string): Folder {
  let folder = tree.get(path);
  if (!folder) {
    folder = { dirs: new Set(), files: [], size: 0, count: 0 };
    tree.set(path, folder);
  }
  return folder;
}
