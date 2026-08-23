/**
 * Families of files, and the type shown per entry.
 *
 * Ren'Py archives hold the same handful of families over and over: images,
 * audio, scripts, and the rest. Sorting on the extension is enough here and
 * needs no read of the entry itself.
 */

import type { MessageKey } from "./i18n.svelte";

/** Families an entry can fall into. */
export type Kind = "images" | "audio" | "scripts" | "other";

const IMAGE = new Set(["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg", "ico", "avif"]);
const AUDIO = new Set(["ogg", "opus", "mp3", "wav", "flac", "m4a", "aac", "mp2"]);
const SCRIPT = new Set([
  "rpy",
  "rpym",
  "rpyc",
  "rpymc",
  "py",
  "json",
  "txt",
  "csv",
  "xml",
  "yaml",
  "yml",
  "md",
]);

/** Extension of a path, lowercased, empty when it has none. */
export function extensionOf(path: string): string {
  const name = path.slice(path.lastIndexOf("/") + 1);
  const dot = name.lastIndexOf(".");
  return dot <= 0 ? "" : name.slice(dot + 1).toLowerCase();
}

/**
 * Bucket a path belongs to.
 *
 * @param path - Entry path.
 * @returns The family, "other" standing for anything that is none of the three.
 */
export function kindOf(path: string): Kind {
  const extension = extensionOf(path);
  if (IMAGE.has(extension)) {
    return "images";
  }
  if (AUDIO.has(extension)) {
    return "audio";
  }
  if (SCRIPT.has(extension)) {
    return "scripts";
  }
  return "other";
}

/** Catalog key naming a family. */
export function kindKey(kind: Kind): MessageKey {
  return `kind.${kind}` as MessageKey;
}

/** Label shown in the type column: the extension, or a dash. */
export function typeLabel(path: string): string {
  const extension = extensionOf(path);
  return extension === "" ? "-" : extension.toUpperCase();
}

const VIDEO = new Set(["webm", "mp4", "mkv", "avi", "mov", "ogv", "mpg", "mpeg"]);
const FONT = new Set(["ttf", "otf", "woff", "woff2", "fnt"]);
const PACKED = new Set(["rpa", "rpi", "zip", "7z", "rar", "gz", "bz2", "xz"]);
/** Ren'Py bytecode: a compiled script, and the reason it cannot be shown. */
const COMPILED = new Set(["rpyc", "rpymc", "rpyb", "pyc", "pyo"]);

/** What to draw, and what to say, for an entry with nothing to display. */
export type Glyph = {
  /** Name of the icon in Icon.svelte. */
  icon: "music" | "video" | "image" | "font" | "code" | "archive" | "file" | "binary";
  /** Catalog key of the line naming what the file is. */
  label: MessageKey;
};

/**
 * Pick the icon and the wording for a file that has no visual preview.
 *
 * An honest "here is what this is" beats a decoding error: a player of an
 * .ogg is not what is missing, and a .rpyc holds bytecode that RenZip
 * deliberately does not decompile.
 *
 * @param path - Entry path.
 */
export function glyphOf(path: string): Glyph {
  const extension = extensionOf(path);
  if (AUDIO.has(extension)) {
    return { icon: "music", label: "glyph.audio" };
  }
  if (VIDEO.has(extension)) {
    return { icon: "video", label: "glyph.video" };
  }
  if (IMAGE.has(extension)) {
    return { icon: "image", label: "glyph.image" };
  }
  if (FONT.has(extension)) {
    return { icon: "font", label: "glyph.font" };
  }
  if (COMPILED.has(extension)) {
    return { icon: "code", label: extension.startsWith("rpy") ? "glyph.rpyc" : "glyph.pyc" };
  }
  if (PACKED.has(extension)) {
    return { icon: "archive", label: "glyph.archive" };
  }
  if (SCRIPT.has(extension)) {
    return { icon: "file", label: "glyph.text" };
  }
  return { icon: "binary", label: "glyph.binary" };
}
