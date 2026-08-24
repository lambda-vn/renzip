/**
 * Interface translations.
 *
 * The window used to carry both languages at once ("Nom / name"), which cost
 * every label twice the room and still read as neither language. One catalog,
 * one language at a time, chosen in the preferences.
 *
 * Both wordings sit on the same line here on purpose: a key whose French and
 * English drift apart is visible at a glance, and adding a language means
 * adding a field rather than hunting through a second file.
 */

/** Languages the interface is available in. */
export const LANGUAGES = ["fr", "en"] as const;

/** One of those languages. */
export type Language = (typeof LANGUAGES)[number];

/** Every string the window shows. */
const MESSAGES = {
  "lang.fr": { fr: "Francais", en: "French" },
  "lang.en": { fr: "Anglais", en: "English" },

  "title.back": { fr: "Retour", en: "Back" },
  "title.settings": { fr: "Préférences", en: "Preferences" },
  "title.create": { fr: "Nouvelle archive", en: "New archive" },
  "title.app": { fr: "RenZip", en: "RenZip" },
  "title.skipToContent": { fr: "Aller au contenu", en: "Skip to content" },

  "dialog.openArchive": { fr: "Ouvrir une archive", en: "Open an archive" },
  "dialog.source": { fr: "Dossier à empaqueter", en: "Folder to pack" },
  "dialog.saveArchive": { fr: "Enregistrer l'archive", en: "Save the archive" },
  "dialog.extractTo": { fr: "Extraire vers", en: "Extract to" },

  "create.dest": { fr: "Fichier de sortie", en: "Output file" },
  "create.options": { fr: "Options", en: "Options" },
  "create.versionAria": { fr: "Version du format à écrire", en: "Format version to write" },
  "create.root": {
    fr: "Le dossier choisi devient la racine de l'archive.",
    en: "The chosen folder becomes the root of the archive.",
  },
  "create.none": { fr: "Aucun dossier choisi pour l'instant.", en: "No folder chosen yet." },

  "browse.filter": { fr: "Rechercher un fichier…", en: "Search for a file…" },
  "browse.filter.label": {
    fr: "Rechercher dans les chemins de l'archive",
    en: "Search the archive paths",
  },
  "browse.open": { fr: "Ouvrir", en: "Open" },
  "browse.create": { fr: "Nouvelle archive", en: "New archive" },
  "browse.welcome": {
    fr: "Ouvrez une archive .rpa, ou déposez-en une sur la fenêtre.",
    en: "Open a .rpa archive, or drop one on the window.",
  },
  "browse.content": { fr: "Contenu", en: "Content" },
  "browse.expandAll": { fr: "Tout déplier", en: "Expand all" },
  "browse.collapseAll": { fr: "Tout replier", en: "Collapse all" },
  "browse.checkAll": { fr: "Tout cocher", en: "Check all" },
  "browse.uncheckAll": { fr: "Tout décocher", en: "Uncheck all" },
  "browse.column.name": { fr: "Nom", en: "Name" },
  "browse.column.type": { fr: "Type", en: "Type" },
  "browse.column.size": { fr: "Taille", en: "Size" },
  "browse.column.offset": { fr: "Offset", en: "Offset" },
  "browse.noMatch": {
    fr: "Aucun fichier ne correspond à la recherche.",
    en: "No file matches the search.",
  },
  "browse.emptyArchive": { fr: "Cette archive est vide.", en: "This archive is empty." },
  "browse.tree": { fr: "Contenu de l'archive", en: "Archive content" },
  "browse.folder": { fr: "dossier", en: "folder" },
  "browse.folderAria": { fr: "Dossier {name}", en: "Folder {name}" },
  "browse.extracted": {
    fr: "{size} extraits · archive de {file}",
    en: "{size} extracted · {file} archive",
  },
  "browse.indexValid": { fr: "Index validé", en: "Index valid" },

  "menu.extract": { fr: "Extraire…", en: "Extract…" },
  "menu.copy": { fr: "Copier le nom", en: "Copy name" },
  "menu.open": { fr: "Ouvrir", en: "Open" },
  "menu.open.hint": { fr: "copie temporaire", en: "temporary copy" },
  "menu.openWith": { fr: "Ouvrir avec…", en: "Open with…" },
  "menu.check": { fr: "Cocher", en: "Check" },
  "menu.uncheck": { fr: "Décocher", en: "Uncheck" },
  "menu.label": { fr: "Actions sur {name}", en: "Actions on {name}" },

  "panel.title": { fr: "Sélection", en: "Selection" },
  "panel.empty": {
    fr: "Choisissez un fichier dans la liste pour l'inspecter.",
    en: "Pick a file in the list to inspect it.",
  },
  "panel.reading": { fr: "Lecture…", en: "Reading…" },
  "panel.showPreview": { fr: "Afficher l'aperçu", en: "Show preview" },
  "panel.tooBig": { fr: "Trop volumineux pour l'aperçu.", en: "Too large to preview." },
  "panel.previewOf": { fr: "Aperçu de {name}", en: "Preview of {name}" },
  "panel.path": { fr: "Chemin", en: "Path" },
  "panel.type": { fr: "Type", en: "Type" },
  "panel.size": { fr: "Taille", en: "Size" },
  "panel.offset": { fr: "Offset", en: "Offset" },
  "panel.stored": { fr: "Stocké", en: "Stored" },
  "panel.stored.value": { fr: "Sans compression", en: "Uncompressed" },
  "panel.prefix": { fr: "Préfixe", en: "Prefix" },
  "panel.prefix.hint": {
    fr: "Octets stockés dans l'index et non dans les données",
    en: "Bytes stored in the index rather than in the data area",
  },
  "panel.destination": { fr: "Destination", en: "Destination" },
  "panel.destination.pick": { fr: "Choisir un dossier", en: "Choose a folder" },
  "panel.browse": { fr: "Parcourir", en: "Browse" },
  "panel.layout.tree": { fr: "Conserver les chemins", en: "Keep the paths" },
  "panel.layout.flat": { fr: "Tout à plat", en: "Flat" },
  "panel.layout.flatNote": {
    fr: "Deux fichiers de même nom ne peuvent pas coexister : le second est signalé plutôt qu'écrasé.",
    en: "Two files with the same name cannot coexist: the second one is reported rather than overwritten.",
  },
  "panel.extract.none": { fr: "Rien de coché", en: "Nothing checked" },
  "panel.extract.one": { fr: "Extraire ce fichier", en: "Extract this file" },
  "panel.extract.many": { fr: "Extraire ces {n} fichiers", en: "Extract these {n} files" },
  "panel.extractAll": { fr: "Tout extraire ({count})", en: "Extract everything ({count})" },

  "create.drop.title": { fr: "Déposez vos fichiers ici", en: "Drop your files here" },
  "create.drop.titleActive": { fr: "Relâchez pour ajouter", en: "Release to add" },
  "create.drop.sub": {
    fr: "Un dossier de jeu, ou n'importe quel dossier à empaqueter.",
    en: "A game folder, or any folder to pack.",
  },
  "create.drop.subActive": {
    fr: "Le dossier déposé sera empaqueté.",
    en: "The dropped folder will be packed.",
  },
  "create.pick": { fr: "Choisir un dossier…", en: "Choose a folder…" },
  "create.staged": { fr: "Contenu préparé", en: "Staged content" },
  "create.name": { fr: "Nom de l'archive", en: "Archive name" },
  "create.version": { fr: "Version RPA", en: "RPA version" },
  "create.note.30": {
    fr: "RPA-3.0 : le format qu'écrit Ren'Py et que lisent les outils tiers.",
    en: "RPA-3.0: what Ren'Py writes and third party tools read.",
  },
  "create.note.20": {
    fr: "RPA-2.0 : index non obfusqué, pour les moteurs anciens.",
    en: "RPA-2.0: unobfuscated index, for older engines.",
  },
  "create.verify": { fr: "Relire l'archive après création", en: "Verify the archive after writing" },
  "create.input": { fr: "Entrée", en: "Input" },
  "create.output": { fr: "Sortie", en: "Output" },
  "create.output.note": {
    fr: "Le format ne compresse pas : l'archive pèse la somme des fichiers, plus son index.",
    en: "The format does not compress: the archive weighs the sum of its files, plus its index.",
  },
  "create.build": { fr: "Créer l'archive", en: "Create the archive" },
  "create.nothing": {
    fr: "Choisissez un dossier à empaqueter.",
    en: "Choose a folder to pack.",
  },

  "settings.general": { fr: "Général", en: "General" },
  "settings.archives": { fr: "Archives", en: "Archives" },
  "settings.about": { fr: "À propos", en: "About" },
  "settings.sections": { fr: "Sections des préférences", en: "Preferences sections" },
  "settings.general.intro": {
    fr: "Le comportement de la fenêtre elle-même.",
    en: "How the window itself behaves.",
  },
  "settings.archives.intro": {
    fr: "Ce que RenZip écrit par défaut dans une nouvelle archive.",
    en: "What RenZip writes by default into a new archive.",
  },
  "settings.language": { fr: "Langue de l'interface", en: "Interface language" },
  "settings.language.note": {
    fr: "S'applique immédiatement, sans redémarrer la fenêtre.",
    en: "Applies at once, without restarting the window.",
  },
  "settings.theme": { fr: "Thème", en: "Theme" },
  "settings.theme.note": {
    fr: "Par défaut celui du bureau. Le thème sombre garde le même accent.",
    en: "The desktop one by default. The dark theme keeps the same accent.",
  },
  "settings.theme.system": { fr: "Système", en: "System" },
  "settings.theme.light": { fr: "Clair", en: "Light" },
  "settings.theme.dark": { fr: "Sombre", en: "Dark" },
  "settings.autoPreview": { fr: "Aperçu automatique", en: "Automatic preview" },
  "settings.autoPreview.note": {
    fr: "Lit l'entrée dès qu'elle est sélectionnée. À couper sur une archive posée sur un disque lent.",
    en: "Reads the entry as soon as it is picked. Turn it off for an archive on a slow disk.",
  },
  "settings.version": { fr: "Version proposée à la création", en: "Version offered when creating" },
  "settings.version.note": {
    fr: "Version présélectionnée dans l'écran « Nouvelle archive ». RPA-3.0 est ce qu'écrit Ren'Py ; RPA-2.0 sert aux moteurs anciens.",
    en: "Version preselected on the \"New archive\" screen. RPA-3.0 is what Ren'Py writes; RPA-2.0 is for older engines.",
  },
  "settings.verify": { fr: "Relire l'archive après création", en: "Verify the archive after writing" },
  "settings.verify.note": {
    fr: "Rouvre le fichier écrit, décode son index et compare ce qu'il annonce à ce qui a été empaqueté.",
    en: "Reopens the written file, decodes its index and compares what it announces with what was packed.",
  },
  "settings.reset": { fr: "Réinitialiser", en: "Reset" },

  "about.tagline": {
    fr: "Gestionnaire d'archives Ren'Py",
    en: "Ren'Py archive manager",
  },
  "about.version": { fr: "Version {version}", en: "Version {version}" },
  "about.stack": {
    fr: "Cœur Rust · interface Svelte et TypeScript · Tauri",
    en: "Rust core · Svelte and TypeScript interface · Tauri",
  },
  "about.reads": { fr: "Lit", en: "Reads" },
  "about.writes": { fr: "Écrit", en: "Writes" },
  "about.license": { fr: "Licence", en: "License" },
  "about.rpyc": {
    fr: "Les scripts compilés (.rpyc) sont extraits tels quels, jamais décompilés.",
    en: "Compiled scripts (.rpyc) are extracted as they are, never decompiled.",
  },

  "kind.images": { fr: "Images", en: "Images" },
  "kind.audio": { fr: "Audio", en: "Audio" },
  "kind.scripts": { fr: "Scripts", en: "Scripts" },
  "kind.other": { fr: "Autres", en: "Other" },

  "glyph.audio": { fr: "Fichier audio", en: "Audio file" },
  "glyph.video": { fr: "Fichier vidéo", en: "Video file" },
  "glyph.image": { fr: "Image", en: "Image" },
  "glyph.font": { fr: "Police de caractères", en: "Font" },
  "glyph.rpyc": {
    fr: "Script Ren'Py compilé (bytecode, non décompilé)",
    en: "Compiled Ren'Py script (bytecode, not decompiled)",
  },
  "glyph.pyc": { fr: "Bytecode Python", en: "Python bytecode" },
  "glyph.archive": { fr: "Archive", en: "Archive" },
  "glyph.text": { fr: "Fichier texte", en: "Text file" },
  "glyph.binary": { fr: "Données binaires", en: "Binary data" },

  "status.archiveOpened": { fr: "{name} ouverte.", en: "{name} opened." },
  "status.copied": { fr: "Nom copié : {name}", en: "Name copied: {name}" },
  "status.opened": { fr: "Ouvert depuis {path}", en: "Opened from {path}" },
  "status.chooser": {
    fr: "Aucune application associée : choisissez-en une pour {path}",
    en: "No application is associated: pick one for {path}",
  },
  "status.revealed": {
    fr: "Aucune application associée à ce type : le dossier a été ouvert sur {path}",
    en: "No application is associated with this type: the folder was opened on {path}",
  },
  "status.cancelled": { fr: ", interrompu", en: ", interrupted" },
  "status.warnLabel": { fr: "Attention", en: "Warning" },
  "status.refreshed": { fr: "Archive actualisée.", en: "Archive refreshed." },
  "status.built": {
    fr: "Archive écrite : {count} ({size}) en {version}{cancelled}.",
    en: "Archive written: {count} ({size}) in {version}{cancelled}.",
  },
  "status.verified": { fr: " Relecture conforme.", en: " Verified." },
  "status.verifyFailed": {
    fr: " Relecture : {reason}",
    en: " Verification: {reason}",
  },
  "status.verifyMismatch": {
    fr: "l'archive annonce {found} pour {expected} empaquetés",
    en: "the archive announces {found} against {expected} packed",
  },
  "status.progress.extract": { fr: "Extraction en cours", en: "Extracting" },
  "status.progress.build": { fr: "Création en cours", en: "Writing" },
  "status.interrupt": { fr: "Interrompre", en: "Stop" },
  "status.skippedDetail": {
    fr: "Détail des entrées non traitées ({count})",
    en: "Untouched entries in detail ({count})",
  },
  "status.multiSegment": {
    fr: "{count} décrite(s) par plusieurs blocs : seul le premier est lu, comme le fait Ren'Py.",
    en: "{count} described by several blocks: only the first is read, as Ren'Py does.",
  },
  "status.notAnArchive": {
    fr: "Ce fichier n'est pas une archive .rpa.",
    en: "This file is not a .rpa archive.",
  },

  "count.entry.one": { fr: "{n} entrée", en: "{n} entry" },
  "count.entry.other": { fr: "{n} entrées", en: "{n} entries" },
  "count.file.one": { fr: "{n} fichier", en: "{n} file" },
  "count.file.other": { fr: "{n} fichiers", en: "{n} files" },
  "count.checked.one": { fr: "{n} fichier coché", en: "{n} file checked" },
  "count.checked.other": { fr: "{n} fichiers cochés", en: "{n} files checked" },
  "count.selected.one": { fr: "fichier sélectionné", en: "file selected" },
  "count.selected.other": { fr: "fichiers sélectionnés", en: "files selected" },
  "count.extracted.one": { fr: "{n} fichier extrait", en: "{n} file extracted" },
  "count.extracted.other": { fr: "{n} fichiers extraits", en: "{n} files extracted" },
  "count.packed.one": { fr: "{n} fichier", en: "{n} file" },
  "count.packed.other": { fr: "{n} fichiers", en: "{n} files" },
  "count.rejected.one": { fr: "{n} entrée refusée", en: "{n} entry refused" },
  "count.rejected.other": { fr: "{n} entrées refusées", en: "{n} entries refused" },
  "count.ignored.one": { fr: "{n} entrée ignorée", en: "{n} entry skipped" },
  "count.ignored.other": { fr: "{n} entrées ignorées", en: "{n} entries skipped" },
  "count.other.one": { fr: "+ {n} autre fichier", en: "+ {n} other file" },
  "count.other.other": { fr: "+ {n} autres fichiers", en: "+ {n} other files" },
} as const;

/** Every key the catalog holds. */
export type MessageKey = keyof typeof MESSAGES;

/** Keys that come in a singular and a plural form. */
type StemOf<Key> = Key extends `${infer Stem}.one` ? Stem : never;

/** Every key that has a .one and a .other form, without that suffix. */
type PluralStem = StemOf<MessageKey>;

/**
 * Language the window is currently drawn in.
 *
 * State, not a constant: reading it inside [`t`] is what makes every label
 * redraw when the preference changes, without a reload.
 */
let language = $state<Language>("fr");

/** Read the current language. */
export function currentLanguage(): Language {
  return language;
}

/** Switch the interface to another language. */
export function setLanguage(next: Language): void {
  language = next;
  document.documentElement.lang = next;
}

/**
 * Pick the language a fresh installation starts in.
 *
 * The desktop's own language decides, since nobody chose anything yet;
 * anything that is not French lands on English.
 */
export function detectLanguage(): Language {
  return navigator.language.toLowerCase().startsWith("fr") ? "fr" : "en";
}

/**
 * Translate one key.
 *
 * @param key - Key of the catalog.
 * @param params - Values for the {placeholders} of the message.
 */
export function t(key: MessageKey, params: Record<string, string | number> = {}): string {
  const message = MESSAGES[key][language];
  return message.replace(/\{(\w+)\}/g, (whole, name: string) =>
    name in params ? String(params[name]) : whole,
  );
}

/**
 * Translate a counted key, picking its singular or its plural.
 *
 * French counts zero as a singular and English does not, which is the whole
 * reason this is not a ternary at each call site.
 *
 * @param stem - Key without its .one or .other suffix.
 * @param n - How many.
 * @param params - Values for the other {placeholders} of the message.
 */
export function tn(
  stem: PluralStem,
  n: number,
  params: Record<string, string | number> = {},
): string {
  const isOne = language === "fr" ? n < 2 : n === 1;
  const key = `${stem}.${isOne ? "one" : "other"}` as MessageKey;
  return t(key, { n: formatNumber(n), ...params });
}

/** Format a number the way the current language writes it. */
export function formatNumber(value: number): string {
  return value.toLocaleString(language === "fr" ? "fr-FR" : "en-US");
}

/** The locale tag matching the current language. */
export function currentLocale(): string {
  return language === "fr" ? "fr-FR" : "en-US";
}
