/** Small formatting helpers shared by the views. */

import type { CommandError, PathRejection, Reason } from "./api";
import { currentLanguage, currentLocale, formatNumber, t } from "./i18n.svelte";

/**
 * Details of a reason or an error, ready to go into a message: numbers are
 * written the way the interface language writes them.
 *
 * @param details - Fields that come with a code, the code itself excluded.
 */
function localized(details: Record<string, string | number>): Record<string, string | number> {
  const values: Record<string, string | number> = {};
  for (const [name, value] of Object.entries(details)) {
    values[name] = typeof value === "number" ? formatNumber(value) : value;
  }
  return values;
}

/**
 * Say why an entry path was refused, in the interface language.
 *
 * @param rejection - What the core found wrong with the path.
 */
function describePathRejection(rejection: PathRejection): string {
  return t(`path.${rejection.kind}`, "detail" in rejection ? { detail: rejection.detail } : {});
}

/**
 * Say why a file was left out or refused, in the interface language.
 *
 * The core sends a code and its details rather than a sentence; every code
 * has its own message in the catalog, which the compiler checks through the
 * template keys below.
 *
 * @param reason - Reason reported by the core.
 */
export function describeReason(reason: Reason): string {
  switch (reason.code) {
    case "io":
      return t(`reason.io.${reason.action}`, { path: reason.path, error: reason.error });
    case "unsafe_path":
    case "unusable_name":
      return t(`reason.${reason.code}`, {
        rejection: describePathRejection(reason.rejection),
      });
    default: {
      const { code, ...details } = reason;
      return t(`reason.${code}`, localized(details));
    }
  }
}

/**
 * Say why a command failed, in the interface language.
 *
 * A backend error arrives as a code and its details, like a reason; anything
 * else a call can reject with (a plugin, the browser itself) is shown as it
 * comes.
 *
 * @param error - Whatever the failed call rejected with.
 */
export function describeError(error: unknown): string {
  if (typeof error !== "object" || error === null || !("code" in error)) {
    return String(error);
  }
  const failure = error as CommandError;
  switch (failure.code) {
    case "corrupt_index": {
      const { kind, ...details } = failure.problem;
      return t("error.corrupt_index", { problem: t(`index.${kind}`, localized(details)) });
    }
    case "truncated": {
      const { kind, ...details } = failure.truncation;
      return t(`error.truncated.${kind}`, localized(details));
    }
    case "invalid_destination":
    case "invalid_source":
      return failure.error === null
        ? t(`error.${failure.code}.not_a_directory`, { path: failure.path })
        : t(`error.${failure.code}`, { path: failure.path, error: failure.error });
    case "entry_skipped":
      return t("error.entry_skipped", {
        entry: failure.entry,
        reason: describeReason(failure.reason),
      });
    default: {
      const { code, ...details } = failure;
      return t(`error.${code}`, localized(details));
    }
  }
}

const UNITS = {
  fr: ["o", "Kio", "Mio", "Gio", "Tio"],
  en: ["B", "KiB", "MiB", "GiB", "TiB"],
};

/**
 * Render a byte count in the unit that keeps it readable.
 *
 * @param bytes - Size to render.
 * @returns The size with its unit, for example "1,4 Mio" or "1.4 MiB".
 */
export function formatSize(bytes: number): string {
  const units = UNITS[currentLanguage()];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit += 1;
  }
  const digits = unit === 0 || value >= 100 ? 0 : 1;
  return `${value.toLocaleString(currentLocale(), { maximumFractionDigits: digits })} ${units[unit]}`;
}
