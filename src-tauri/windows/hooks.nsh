; Tauri's installer only writes .rpa and the RenZip.Archive ProgID. When
; the user picks RenZip through "Open with > Choose another app", Windows
; records the default as Applications\renzip.exe instead, with the path of
; the exe picked at that moment. Once that copy is gone (an older install
; elsewhere, an uninstall during an upgrade) the default points nowhere and
; Windows resets it. Pointing that key at this install keeps such a default
; working and leaves a single RenZip in the Open with list.

!macro NSIS_HOOK_POSTINSTALL
  WriteRegStr SHCTX "Software\Classes\Applications\${MAINBINARYNAME}.exe" "FriendlyAppName" "${PRODUCTNAME}"
  WriteRegStr SHCTX "Software\Classes\Applications\${MAINBINARYNAME}.exe\SupportedTypes" ".rpa" ""
  WriteRegStr SHCTX "Software\Classes\Applications\${MAINBINARYNAME}.exe\shell\open\command" "" '"$INSTDIR\${MAINBINARYNAME}.exe" "%1"'
  ; SHCNE_ASSOCCHANGED, SHCNF_FLUSH: Explorer otherwise keeps the
  ; associations it cached before the install.
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0x1000, p 0, p 0)'
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  DeleteRegKey SHCTX "Software\Classes\Applications\${MAINBINARYNAME}.exe"
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0x1000, p 0, p 0)'
!macroend
