/**
 * Preferences, kept in the browser storage of the window.
 *
 * Only settings that change what the application really does are here. The
 * design's Preferences screen also offered a compression setting, which has
 * nothing to act on: an .rpa stores its files uncompressed.
 */

import { detectLanguage, LANGUAGES, type Language } from "./i18n.svelte";

const STORAGE_KEY = "renzip.settings";

/** Archive versions this application can write, in the order they are shown. */
export const WRITABLE_VERSIONS = ["RPA-2.0", "RPA-3.0"] as const;

/** Everything the user can set. */
export type Settings = {
  /** Language the interface is drawn in. */
  language: Language;
  /** "systeme" follows the desktop, the other two force a theme. */
  theme: "systeme" | "clair" | "sombre";
  /** Version proposed when creating an archive. */
  defaultVersion: (typeof WRITABLE_VERSIONS)[number];
  /** Read and show an entry as soon as it is selected. */
  autoPreview: boolean;
  /** Reopen a freshly built archive and check it against what was packed. */
  verifyAfterBuild: boolean;
};

/** Settings a fresh installation starts with. */
export const DEFAULTS: Settings = {
  // The desktop decides until the user says otherwise.
  language: detectLanguage(),
  theme: "systeme",
  defaultVersion: "RPA-3.0",
  autoPreview: true,
  verifyAfterBuild: true,
};

/**
 * Read the stored settings, falling back to the defaults field by field.
 *
 * A stored value of the wrong shape is ignored rather than trusted: the store
 * survives application updates, and a setting that disappeared should not
 * come back as undefined somewhere in the interface.
 */
export function loadSettings(): Settings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return { ...DEFAULTS };
    }
    const stored = JSON.parse(raw) as Partial<Settings>;
    return {
      language: LANGUAGES.includes(stored.language as Language)
        ? (stored.language as Language)
        : DEFAULTS.language,
      theme:
        stored.theme === "clair" || stored.theme === "sombre" || stored.theme === "systeme"
          ? stored.theme
          : DEFAULTS.theme,
      defaultVersion: WRITABLE_VERSIONS.includes(
        stored.defaultVersion as (typeof WRITABLE_VERSIONS)[number],
      )
        ? (stored.defaultVersion as (typeof WRITABLE_VERSIONS)[number])
        : DEFAULTS.defaultVersion,
      autoPreview:
        typeof stored.autoPreview === "boolean" ? stored.autoPreview : DEFAULTS.autoPreview,
      verifyAfterBuild:
        typeof stored.verifyAfterBuild === "boolean"
          ? stored.verifyAfterBuild
          : DEFAULTS.verifyAfterBuild,
    };
  } catch {
    return { ...DEFAULTS };
  }
}

/** Persist the settings and apply the ones the document carries. */
export function saveSettings(settings: Settings): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(settings));
  } catch {
    // A window running without storage still works, it just forgets.
  }
  applyTheme(settings.theme);
}

/**
 * Put the chosen theme on the root element.
 *
 * "systeme" removes the attribute so the media query in app.css decides.
 */
export function applyTheme(theme: Settings["theme"]): void {
  const root = document.documentElement;
  if (theme === "systeme") {
    root.removeAttribute("data-theme");
  } else {
    root.setAttribute("data-theme", theme === "sombre" ? "dark" : "light");
  }
}
