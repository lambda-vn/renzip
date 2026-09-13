/** Small formatting helpers shared by the views. */

import type { PathRejection, Reason } from "./api";
import { currentLanguage, currentLocale, formatNumber, t } from "./i18n.svelte";

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
      const values: Record<string, string | number> = {};
      for (const [name, value] of Object.entries(details)) {
        values[name] = typeof value === "number" ? formatNumber(value) : value;
      }
      return t(`reason.${code}`, values);
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
