; Hooks into Tauri's NSIS template. The app wires the Claude Code hooks itself at every start;
; only the uninstaller can take them out again, or Claude Code keeps calling a hook that is gone.

!macro NSIS_HOOK_PREUNINSTALL
  ; An update runs the old uninstaller with /UPDATE: the hooks stay, the new copy rewires them
  ${If} $UpdateMode <> 1
    nsExec::Exec '"$INSTDIR\${MAINBINARYNAME}.exe" uninstall-hooks'
    Pop $0
  ${EndIf}
!macroend
