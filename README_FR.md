<div align="center">
  <img src="src/assets/logo-256.png" alt="" width="96">
  <h1>RenZip</h1>

[![Derniere version](https://img.shields.io/github/v/release/lambda-vn/renzip?label=version)](https://github.com/lambda-vn/renzip/releases/latest)
[![CI](https://img.shields.io/github/actions/workflow/status/lambda-vn/renzip/ci.yml?branch=main&label=CI)](https://github.com/lambda-vn/renzip/actions/workflows/ci.yml)
[![Build](https://img.shields.io/github/actions/workflow/status/lambda-vn/renzip/build.yml?label=build)](https://github.com/lambda-vn/renzip/actions/workflows/build.yml)
[![Licence : CeCILL-2.1](https://img.shields.io/badge/licence-CeCILL--2.1-blue)](LICENSE.md)
[![Rust + Tauri 2](https://img.shields.io/badge/rust%20%2B%20tauri-2-orange)](https://v2.tauri.app)
[![Plateformes : Windows, macOS, Linux](https://img.shields.io/badge/plateformes-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey)](https://github.com/lambda-vn/renzip/releases/latest)

</div>

Gestionnaire d'archives **Ren'Py `.rpa`** pour le bureau, dans l'esprit de
7-Zip ou WinRAR mais dédié au format dans lequel les visual novels livrent
leurs ressources. Ouvrir une archive, voir ce qu'elle contient, en extraire la
totalité ou trois fichiers, et reconstruire une archive à partir d'un dossier.
Sans ligne de commande, sans script Python, sans `.bat` téléchargé sur un
forum.

Version anglaise : [README.md](README.md).

## Fonctionnalités

- Ouverture par sélection du fichier, ou par glisser-déposer sur la fenêtre.
- Un seul arbre pour toute l'archive : les dossiers se déplient sur place, une
  case à cocher sur n'importe quelle ligne construit la sélection, et cocher
  un dossier prend tout ce qu'il contient. La liste est virtualisée : cinquante
  mille entrées défilent comme dix.
- Une recherche qui conserve le chemin complet de ce qu'elle trouve, pour
  qu'une correspondance à trois dossiers de profondeur reste lisible comme
  telle.
- Aperçu des entrées texte et des formats d'image courants, directement depuis
  l'archive, sans rien extraire au préalable.
- Menu contextuel sur une ligne : extraire, copier le nom de l'entrée, ouvrir
  une copie temporaire avec ce que le bureau associe à ce type de fichier.
- Extraction d'une sélection ou de la totalité, en conservant l'arborescence
  de l'archive ou à plat, avec une barre de progression honnête et un bouton
  d'annulation.
- Création d'une archive à partir d'un dossier, avec relecture facultative
  après écriture pour la comparer à ce qui a été empaqueté.
- Interface française et anglaise, choisie dans les paramètres, devinée depuis
  le bureau au premier lancement. Thèmes clair et sombre.
- Utilisable entièrement au clavier, selon le motif ARIA « arbre » : les
  flèches parcourent et déplient, l'espace coche, Entrée ouvre.

## Formats gérés

Un `.rpa` est un conteneur, pas un compresseur : seul son index est compressé,
les données des fichiers sont stockées telles quelles. Une extraction
sélective se réduit donc à un `seek` et un `read`, ce qui rend instantané
l'aperçu d'une entrée au sein d'une archive de quatre gigaoctets.

| Version | Lecture | Écriture | Note |
|---|---|---|---|
| RPA-1.0 | oui | non | L'index se trouve dans un fichier `.rpi` voisin |
| RPA-2.0 | oui | oui | Offsets et longueurs en clair |
| RPA-3.0 | oui | oui | Offsets et longueurs obfusqués par XOR avec une clé |
| RPA-3.2 | oui | non | Un champ d'en-tête supplémentaire avant la clé |
| RPA-4.0 | oui | non | Même structure que la 3.0, seul le marqueur change |

Ni la 3.2 ni la 4.0 ne sont proposées en écriture. La forme de leur en-tête
n'est plus un mystère : celle de la 3.2 est offset, un champ supplémentaire
inutilisé, puis la clé (confirmé par le développeur de Zeliria Sanctuary, le
seul jeu connu à l'utiliser, et par Shizmob/rpatool#9) ; celle de la 4.0 est
la même que celle de la 3.0 (Lattyware/unrpa#22) — cette bibliothèque pourrait
donc en écrire. Mais aucun Ren'Py standard ne les lit : un jeu qui en ouvre
une embarque un chargeur modifié, donc écrire une archive que le moteur
lui-même ne peut pas lire serait pire que refuser, quoi que dise l'en-tête.
L'écran des paramètres le dit, plutôt que de masquer l'option.

Deux choses que le format ne contient tout simplement pas, et que la fenêtre
n'affiche donc pas : une somme de contrôle et une date de modification. Ni une
taille compressée, puisque rien n'est compressé.

## Pile technique

Rust / Tauri 2 / Svelte 5 / TypeScript / Tailwind 4.

Le lecteur et l'écrivain vivent dans `renzip-core`, une bibliothèque Rust
ordinaire sans la moindre dépendance à Tauri. C'est une règle dure : elle rend
le travail sur le format testable sans ouvrir de fenêtre, et laisse la porte
ouverte à un binaire en ligne de commande plus tard.

## Prérequis

- Rust stable, 1.77.2 ou plus récent, ce que réclame Tauri 2.
- Node 20 ou plus récent.
- Les dépendances système de Tauri pour votre plateforme, listées dans les
  [prérequis Tauri](https://v2.tauri.app/start/prerequisites/). Sur Debian et
  Ubuntu il s'agit de WebKitGTK et d'une poignée de paquets de développement ;
  sur Windows, de WebView2, que Windows 11 possède déjà.
- Python 3, uniquement pour régénérer les fixtures de test et l'icône. Ni la
  compilation ni l'application n'en ont besoin.

## Installation

```bash
git clone https://github.com/lambda-vn/renzip.git
cd renzip
npm install
```

## Lancement

```bash
npm run tauri dev
```

Vite sert le frontend et Tauri ouvre la fenêtre dessus. Déposez un `.rpa` sur
cette fenêtre, ou passez par le bouton.

## Développement

Après chaque modification, les quatre contrôles doivent passer, dans l'ordre
où le CI les joue :

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --workspace -- -D warnings
cargo test --workspace
npm run build
```

`npm run build` n'est pas seulement une compilation : il lance d'abord
`svelte-check`, qui est à la fois le vérificateur de types et le linter
d'accessibilité du frontend.

`pre-commit install` branche les mêmes contrôles, plus le hook de message de
commit.

### Tests

Tests unitaires à côté du code, tests d'intégration dans
`crates/renzip-core/tests/`. Deux d'entre eux comptent plus que les autres :

- `read_fixtures.rs` ouvre chaque archive de `tests/fixtures/` et compare ce
  qui en sort à `expected.json`, produit par un script Python qui lit les
  mêmes archives comme le fait Ren'Py, et non par ce code-ci.
- `roundtrip.rs` empaquette un dossier, ouvre le résultat, l'extrait, et
  vérifie que les fichiers sont identiques bit à bit. C'est le test qui
  attrape une erreur de `prefix` ou de borne.

Les fixtures couvrent une RPA-2.0, une RPA-3.0, une RPA-3.2, une RPA-4.0, une
1.0 dont les fichiers sont découpés en plusieurs blocs, une archive écrite
comme le fait l'archiveur de Ren'Py lui-même, une dont les entrées portent un
préfixe, une vide, une tronquée, une déclarant des octets au-delà de la fin du
fichier, une dont les entrées concatènent plusieurs segments, une avec un
segment qu'aucun lecteur ne peut interpréter non plus, et une pleine de
chemins malveillants.

```bash
cargo test --workspace
python scripts/make_fixtures.py           # réécrit tests/fixtures/
python scripts/read_archive.py x.rpa      # relit une archive avec pickle, comme Ren'Py
```

### Construire un binaire de bureau

```bash
npm run tauri build
```

Construit pour le système sur lequel la commande tourne : Tauri se lie au
webview natif, donc rien ne se compile de façon croisée entre les trois
plateformes. Les trois sont construites par le workflow Build, un runner
chacune, et macOS en bundle universel. Les binaires ne sont pas signés :
SmartScreen et Gatekeeper afficheront un avertissement.

## Organisation du projet

```
crates/renzip-core/    Le format, sans dépendance à Tauri
  src/header.rs        Détection de version et lecture de l'en-tête
  src/index.rs         zlib et pickle, dans les deux sens
  src/archive.rs       Ouverture, listing, lecture, extraction
  src/builder.rs       Parcours d'un dossier et écriture d'une archive
  src/path.rs          Assainissement des chemins et confinement
  tests/               Lecture des fixtures et test de boucle
src-tauri/             Commandes, fenêtre, événements de progression
  src/desktop.rs       Confier un fichier au bureau, par plateforme
  src/preview.rs       Décider quoi afficher pour une entrée
  src/state.rs         Archives ouvertes et leurs dossiers temporaires
src/                   Frontend Svelte 5
  lib/EntryTree.svelte L'unique liste de la fenêtre principale
  lib/VirtualList.svelte  Virtualisation écrite à la main, sans dépendance
  lib/i18n.svelte.ts   Français et anglais
scripts/               Fixtures, icône, hook de message de commit
tests/fixtures/        Archives à lire, et ce qu'elles doivent produire
```

## Sécurité

Un `.rpa` est un fichier d'origine inconnue, et cette application le traite
comme hostile. En résumé :

- L'index est un pickle Python. Il est analysé par `serde-pickle`, qui lit des
  valeurs et ne construit jamais l'objet que le flux réclame, et n'est accepté
  que dans la seule forme que définit le format.
- Les chemins des entrées sont assainis, puis la destination est vérifiée sur
  le chemin canonique juste avant écriture. Une entrée refusée est signalée et
  ignorée, jamais fatale au reste de l'extraction.
- Les offsets et les longueurs sont validés contre la taille réelle du fichier
  avant toute allocation. Rien ne charge une archive en entier.
- La fenêtre ne dispose d'aucune commande prenant un chemin pour l'ouvrir.

La politique complète, et ce qui en est délibérément exclu, se trouve dans
[SECURITY.md](SECURITY.md).

## Commits

Conventional Commits, validés par pre-commit. Voir
[COMMIT_CONVENTION.md](COMMIT_CONVENTION.md) pour les types, les portées et
les exemples.

## Licence

Distribué sous licence CeCILL v2.1, une licence libre française compatible
avec la GNU GPL. Voir [LICENSE.md](LICENSE.md).

## Avertissement

RenZip n'est ni affilié, ni soutenu, ni sponsorisé par Ren'Py. Le nom sert
uniquement à désigner le moteur dont cet outil lit et écrit le format
d'archive. RenZip ne décompile pas le bytecode `.rpyc`, et extraire une
archive ne donne aucun droit de redistribuer ce qu'elle contient.
