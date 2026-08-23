/** Small formatting helpers shared by the views. */

import { currentLanguage, currentLocale } from "./i18n.svelte";

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
