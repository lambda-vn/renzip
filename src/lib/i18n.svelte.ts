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
  "menu.remove": { fr: "Retirer", en: "Remove" },
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
  "create.drop.multiple": {
    fr: "Déposez un seul dossier : celui qui contient tout ce que vous voulez empaqueter. Ce qui ne doit pas y figurer peut être retiré une fois le dossier chargé.",
    en: "Drop a single folder: the one containing everything you want to pack. Anything that should not be in it can be dropped once the folder is loaded.",
  },
  "create.archivesSkipped.title": { fr: "Fichiers ignorés", en: "Files skipped" },
  "create.archivesSkipped.body": {
    fr: "Ren'Py n'ouvre jamais une archive rangée dans une autre. Ignorés dans ce dossier : {count}.\n\n{list}",
    en: "Ren'Py never opens an archive packed inside another. Skipped in this folder: {count}.\n\n{list}",
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
    fr: "Archive écrite dans {path} : {count} ({size}) en {version}.",
    en: "Archive written to {path}: {count} ({size}) in {version}.",
  },
  "status.buildCancelled": {
    fr: "Création annulée : rien n'a été écrit dans {path}. Si une archive existait déjà à cet emplacement, elle est intacte.",
    en: "Build cancelled: nothing was written to {path}. If an archive already existed there, it is untouched.",
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
  "status.notAnArchive": {
    fr: "Ce fichier n'est pas une archive .rpa.",
    en: "This file is not a .rpa archive.",
  },

  "reason.unreadable_segments": {
    fr: "l'index décrit cette entrée en plusieurs blocs dont au moins un n'est pas une simple paire (offset, longueur) ; Ren'Py ne peut pas la lire non plus",
    en: "index lists several tuples for this entry and at least one is not a plain (offset, length) pair; Ren'Py cannot read this entry either",
  },
  "reason.unsafe_path": { fr: "chemin dangereux : {rejection}", en: "unsafe path: {rejection}" },
  "reason.prefix_too_long": {
    fr: "déclare un préfixe de {prefix} octets dans une entrée de {length} octets",
    en: "declares a {prefix} byte prefix inside a {length} byte entry",
  },
  "reason.past_end": {
    fr: "déclare les octets {start} à {end} d'une archive de {size} octets",
    en: "declares bytes {start}..{end} of a {size} byte archive",
  },
  "reason.segment_past_end": {
    fr: "déclare un bloc aux octets {start} à {end} d'une archive de {size} octets",
    en: "declares a segment at bytes {start}..{end} of a {size} byte archive",
  },
  "reason.range_overflow": {
    fr: "déclare {length} octets à la position {offset}, au-delà de la fin de tout fichier",
    en: "declares {length} bytes at offset {offset}, past the end of any file",
  },
  "reason.segment_overflow": {
    fr: "déclare un bloc de {length} octets à la position {offset}, au-delà de la fin de tout fichier",
    en: "declares a segment of {length} bytes at offset {offset}, past the end of any file",
  },
  "reason.duplicate_path": {
    fr: "chemin en double : « {path} » est déjà dans l'archive",
    en: "duplicate path, '{path}' is already in the archive",
  },
  "reason.no_file_name": { fr: "l'entrée n'a pas de nom de fichier", en: "entry has no file name" },
  "reason.no_parent_directory": {
    fr: "l'entrée n'a pas de dossier parent",
    en: "entry has no parent directory",
  },
  "reason.already_written": {
    fr: "une autre entrée de cette extraction a déjà écrit {path}",
    en: "another entry of this extraction already wrote {path}",
  },
  "reason.outside_destination": { fr: "écrirait en dehors de {root}", en: "would write outside of {root}" },
  "reason.archive_ends_early": {
    fr: "l'archive s'arrête après {written} des {announced} octets annoncés",
    en: "archive ends after {written} of the {announced} bytes announced",
  },
  "reason.destination_of_build": {
    fr: "laissé de côté : c'est la destination de cette création",
    en: "left out: this is the destination of this build",
  },
  "reason.symbolic_link": { fr: "lien symbolique", en: "symbolic link" },
  "reason.archive_in_archive": {
    fr: "fichier d'archive : Ren'Py n'en ouvre jamais une rangée dans une autre",
    en: "archive file: Ren'Py never opens one packed inside another",
  },
  "reason.unusable_name": { fr: "nom inutilisable : {rejection}", en: "unusable name: {rejection}" },
  "reason.io.create": { fr: "impossible de créer {path} : {error}", en: "cannot create {path}: {error}" },
  "reason.io.resolve": { fr: "impossible de résoudre {path} : {error}", en: "cannot resolve {path}: {error}" },
  "reason.io.replace_link": {
    fr: "impossible de remplacer le lien {path} : {error}",
    en: "cannot replace the link at {path}: {error}",
  },
  "reason.io.write": { fr: "impossible d'écrire {path} : {error}", en: "cannot write {path}: {error}" },
  "reason.io.read": { fr: "impossible de lire {path} : {error}", en: "cannot read {path}: {error}" },
  "reason.io.extract": { fr: "impossible d'extraire {path} : {error}", en: "cannot extract {path}: {error}" },
  "reason.io.seek": {
    fr: "impossible de se déplacer dans {path} : {error}",
    en: "cannot seek in {path}: {error}",
  },
  "reason.io.store": { fr: "impossible de stocker {path} : {error}", en: "cannot store {path}: {error}" },
  "path.empty": { fr: "chemin vide", en: "empty path" },
  "path.absolute": { fr: "chemin absolu", en: "absolute path" },
  "path.drive_letter": { fr: "lettre de lecteur", en: "drive letter" },
  "path.traversal": { fr: "remontée hors du dossier (..)", en: "path traversal" },
  "path.reserved_name": { fr: "nom réservé « {detail} »", en: "reserved name '{detail}'" },
  "path.forbidden_character": { fr: "caractère interdit « {detail} »", en: "forbidden character '{detail}'" },
  "path.trailing_dot_or_space": {
    fr: "« {detail} » finit par un point ou une espace",
    en: "component '{detail}' ends with a dot or a space",
  },

  "error.io": { fr: "Accès impossible à {path} : {error}", en: "Cannot access {path}: {error}" },
  "error.unsupported_header": {
    fr: "Ce fichier n'est pas une archive reconnue (en-tête : « {header} »).",
    en: "This file is not a recognized archive (header: '{header}').",
  },
  "error.missing_index_file": {
    fr: "Archive RPA-1.0 sans son fichier d'index : {path} est introuvable.",
    en: "RPA-1.0 archive without its index file: {path} is missing.",
  },
  "error.corrupt_index": {
    fr: "L'index de l'archive est illisible : {problem}.",
    en: "The archive index is unreadable: {problem}.",
  },
  "error.truncated.index_past_end": {
    fr: "L'archive est tronquée : son index est annoncé à la position {offset} d'un fichier de {size} octets.",
    en: "The archive is truncated: its index is announced at offset {offset} of a {size} byte file.",
  },
  "error.truncated.entry_stops_early": {
    fr: "L'archive est tronquée : l'entrée « {entry} » s'arrête après {read} des {length} octets annoncés.",
    en: "The archive is truncated: entry '{entry}' stops after {read} of {length} bytes.",
  },
  "error.entry_not_found": {
    fr: "L'archive ne contient pas d'entrée « {entry} ».",
    en: "The archive has no entry '{entry}'.",
  },
  "error.invalid_destination": {
    fr: "Dossier de destination inutilisable, {path} : {error}",
    en: "Unusable destination folder, {path}: {error}",
  },
  "error.invalid_destination.not_a_directory": {
    fr: "La destination {path} n'est pas un dossier.",
    en: "The destination {path} is not a folder.",
  },
  "error.invalid_source": {
    fr: "Dossier source illisible, {path} : {error}",
    en: "Unreadable source folder, {path}: {error}",
  },
  "error.invalid_source.not_a_directory": {
    fr: "La source {path} n'est pas un dossier.",
    en: "The source {path} is not a folder.",
  },
  "error.unsupported_write_version": {
    fr: "Impossible d'écrire une archive en version {version}.",
    en: "Cannot write archives in version {version}.",
  },
  "error.archive_not_open": {
    fr: "Cette archive a été fermée entre-temps.",
    en: "This archive was closed in the meantime.",
  },
  "error.entry_skipped": {
    fr: "{entry} n'a pas pu être ouvert : {reason}.",
    en: "{entry} could not be opened: {reason}.",
  },
  "error.reveal_failed": {
    fr: "Impossible de montrer le fichier dans son dossier : {error}",
    en: "Cannot show the file in its folder: {error}",
  },
  "error.internal": { fr: "Erreur interne : {error}", en: "Internal error: {error}" },
  "index.inflate": { fr: "décompression impossible ({error})", en: "cannot inflate it ({error})" },
  "index.too_large": {
    fr: "il dépasse {limit} octets une fois décompressé",
    en: "it inflates past {limit} bytes",
  },
  "index.unpickle": { fr: "décodage impossible ({error})", en: "cannot unpickle it ({error})" },
  "index.not_a_dict": { fr: "ce n'est pas un dictionnaire", en: "it is not a dict" },
  "index.key_not_a_string": {
    fr: "une clé n'est pas une chaîne ({key})",
    en: "a key is not a string ({key})",
  },
  "index.not_a_list": {
    fr: "l'entrée « {entry} » n'est pas une liste de tuples",
    en: "entry '{entry}' is not a list of tuples",
  },
  "index.no_tuple": { fr: "l'entrée « {entry} » ne décrit aucun bloc", en: "entry '{entry}' has no tuple" },
  "index.field_count": {
    fr: "l'entrée « {entry} » a {count} champs au lieu de 2 ou 3",
    en: "entry '{entry}' has {count} fields, expected 2 or 3",
  },
  "index.not_a_tuple": {
    fr: "l'entrée « {entry} » n'est pas décrite par un tuple",
    en: "entry '{entry}' is not described by a tuple",
  },
  "index.prefix_not_bytes": {
    fr: "le préfixe de l'entrée « {entry} » n'est pas une chaîne d'octets ({value})",
    en: "entry '{entry}' has a prefix that is not a byte string ({value})",
  },
  "index.not_a_positive_integer": {
    fr: "l'entrée « {entry} » a une position ou une taille qui n'est pas un entier positif ({value})",
    en: "entry '{entry}' has an offset or length that is not a positive integer ({value})",
  },
  "index.pickle": { fr: "encodage impossible ({error})", en: "cannot pickle it ({error})" },
  "index.deflate": { fr: "compression impossible ({error})", en: "cannot deflate it ({error})" },

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
