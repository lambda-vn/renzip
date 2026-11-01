# Associations de fichiers Windows avec Tauri 2

Note réutilisable pour toute app Rust + Tauri 2 qui déclare `bundle.fileAssociations`
et s'installe sous Windows. Elle décrit deux symptômes observés sur RenZip, leur cause
et la correction appliquée.

## Symptômes

1. Clic droit, puis « Ouvrir avec » : l'application apparaît plusieurs fois.
2. Après l'installation d'une nouvelle version, l'application n'est plus le
   programme par défaut de son extension.

## Ce que Tauri écrit réellement

### Installeur NSIS (`*-setup.exe`)

- `installMode` vaut par défaut `currentUser` : installation dans
  `%LOCALAPPDATA%\<Produit>`, registre sous `HKCU` (`SHCTX`).
- La macro `APP_ASSOCIATE` (`FileAssociation.nsh`) écrit seulement :
  - `Software\Classes\.<ext>` : valeur par défaut = le ProgID (`fileAssociations[].name`),
    plus une valeur `<ProgID>_backup` qui contient l'ancienne valeur ;
  - `Software\Classes\<ProgID>` : description, `DefaultIcon`, `shell\open\command`.
- Elle n'écrit ni `Applications\<exe>`, ni `OpenWithProgids`, ni `Capabilities`.
  Elle n'appelle pas `SHChangeNotify` non plus.
- À la désinstallation, `APP_UNASSOCIATE` restaure `_backup` et **supprime le ProgID**.
  Cette étape n'est pas soumise au mode mise à jour.
- Mise à jour manuelle (on lance un nouveau `setup.exe` par-dessus l'ancien) :
  l'installeur exécute l'**ancien** `uninstall.exe`, qui supprime le ProgID, puis il le
  recrée. Avec l'updater Tauri (`/UPDATE`), l'ancien désinstalleur n'est pas lancé.
- Si l'installeur trouve un MSI du même produit (`FindWixInstall`), il le désinstalle.

### Installeur MSI (WiX)

- `perMachine` : installation dans `C:\Program Files\<Produit>`, registre sous `HKLM`.
- Le ProgID et l'extension sont déclarés comme éléments MSI « advertised ».
- Le MSI ne détecte pas une installation NSIS existante.

### Ce que Windows écrit lui-même

- Quand on choisit l'application par « Ouvrir avec > Choisir une autre application »
  (surtout avec « Parcourir »), Windows crée
  `HKCU\Software\Classes\Applications\<exe>\shell\open\command`. Le chemin enregistré
  est celui de l'exe choisi à ce moment-là, et le choix par défaut devient
  `UserChoice\ProgId = Applications\<exe>`.
- `UserChoice` est protégé par un hash : aucun installeur ne peut l'écrire. Seul
  l'utilisateur peut le définir, via l'interface de Windows.
- `MuiCache` mémorise chaque exe lancé, y compris `target\debug\*.exe`. C'est sans
  conséquence.

## Causes

1. **`bundle.targets: "all"`** produit à la fois un MSI et un NSIS. Si on installe l'un
   puis l'autre, on obtient deux chemins et deux entrées : le ProgID NSIS dans `HKCU`
   et `Applications\<exe>` vers `Program Files`.
2. L'entrée `Applications\<exe>` que Windows a créée garde le chemin de l'ancienne
   installation. Quand ce dossier disparaît (MSI désinstallé par le NSIS, ancienne
   version supprimée), le choix par défaut ne pointe plus nulle part et Windows le
   réinitialise.
3. Pendant une mise à jour manuelle, le ProgID est supprimé puis recréé sans que
   l'Explorateur en soit averti. Son cache d'associations peut donc rester périmé.

## Correction

### 1. Un seul installeur Windows

`src-tauri/tauri.windows.conf.json` : Tauri fusionne ce fichier avec `tauri.conf.json`
seulement sous Windows, donc `"all"` reste valable sous Linux et macOS.

```json
{
  "bundle": {
    "targets": ["nsis"],
    "windows": {
      "nsis": {
        "installMode": "currentUser",
        "installerHooks": "./windows/hooks.nsh"
      }
    }
  }
}
```

### 2. Hooks NSIS

`src-tauri/windows/hooks.nsh` : remplacer `.rpa` par l'extension de l'app.
`${MAINBINARYNAME}` et `${PRODUCTNAME}` sont définis par le modèle de Tauri.

```nsis
!macro NSIS_HOOK_POSTINSTALL
  WriteRegStr SHCTX "Software\Classes\Applications\${MAINBINARYNAME}.exe" "FriendlyAppName" "${PRODUCTNAME}"
  WriteRegStr SHCTX "Software\Classes\Applications\${MAINBINARYNAME}.exe\SupportedTypes" ".rpa" ""
  WriteRegStr SHCTX "Software\Classes\Applications\${MAINBINARYNAME}.exe\shell\open\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" "%1"'
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0x1000, p 0, p 0)'
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  DeleteRegKey SHCTX "Software\Classes\Applications\${MAINBINARYNAME}.exe"
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0x1000, p 0, p 0)'
!macroend
```

- `POSTINSTALL` fait pointer `Applications\<exe>` vers l'installation courante. Un
  choix par défaut fait via « Ouvrir avec » survit ainsi aux changements de chemin, et
  les deux entrées désignent le même exe.
- `SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_FLUSH)` force l'Explorateur à relire les
  associations.
- `POSTUNINSTALL` retire la clé lors d'une vraie désinstallation. Pendant une mise à
  jour manuelle, le `POSTINSTALL` du nouvel installeur la recrée aussitôt.

Les autres hooks disponibles sont `NSIS_HOOK_PREINSTALL` et `NSIS_HOOK_PREUNINSTALL`.

### 3. Mises à jour

L'updater Tauri (`tauri-plugin-updater`) passe `/UPDATE`. L'ancien désinstalleur n'est
alors pas exécuté et le ProgID n'est jamais supprimé.

## Réparer une machine déjà touchée (PowerShell, utilisateur courant)

1. Désinstaller toutes les copies de l'application (Paramètres > Applications), MSI
   comme NSIS.
2. Supprimer les restes (adapter `renzip.exe`, `RenZip.Archive`, `.rpa`) :

   ```powershell
   Remove-Item -Recurse -Force 'HKCU:\Software\Classes\Applications\renzip.exe' -ErrorAction SilentlyContinue
   Remove-Item -Recurse -Force 'HKCU:\Software\Classes\RenZip.Archive' -ErrorAction SilentlyContinue
   Remove-ItemProperty 'HKCU:\Software\Classes\.rpa' -Name 'RenZip.Archive_backup' -ErrorAction SilentlyContinue
   Remove-Item -Recurse -Force 'HKCU:\Software\Classes\rpa_auto_file' -ErrorAction SilentlyContinue
   Remove-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.rpa\OpenWithProgids' -Name 'rpa_auto_file' -ErrorAction SilentlyContinue
   ```

   `<ext>_auto_file` est un ProgID que Windows crée lui-même quand on choisit un exe
   via « Parcourir ». Il garde le chemin de l'exe à ce moment-là et survit au
   redémarrage de l'Explorateur.

3. Installer une seule version, construite avec les hooks ci-dessus.
4. Définir le programme par défaut : clic droit sur un fichier, puis « Ouvrir avec >
   Choisir une autre application », sélectionner l'app et cliquer sur « Toujours ». On
   peut aussi passer par Paramètres > Applications > Applications par défaut.

## Vérifier

```powershell
Get-ItemProperty 'HKCU:\Software\Classes\.rpa'
(Get-ItemProperty 'HKCU:\Software\Classes\RenZip.Archive\shell\open\command').'(default)'
(Get-ItemProperty 'HKCU:\Software\Classes\Applications\renzip.exe\shell\open\command').'(default)'
Get-ItemProperty 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.rpa\UserChoiceLatest\ProgId' -ErrorAction SilentlyContinue
```

Les deux commandes doivent pointer vers le même exe, celui de l'installation courante.

## Sources

- `tauri-apps/tauri`, `crates/tauri-bundler/src/bundle/windows/nsis/`
  (`installer.nsi`, `FileAssociation.nsh`, `mod.rs`) et `msi/main.wxs`
- https://v2.tauri.app/distribute/windows-installer/
- https://v2.tauri.app/reference/config/
- https://learn.microsoft.com/en-us/windows/win32/shell/app-registration
- https://learn.microsoft.com/en-us/windows/win32/shell/how-to-include-an-application-on-the-open-with-dialog-box
