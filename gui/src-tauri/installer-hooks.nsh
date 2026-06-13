; Labalaba NSIS installer hooks.
; Wired via bundle.windows.nsis.installerHooks in tauri.conf.json.

!macro NSIS_HOOK_PREUNINSTALL
  ; Stop the daemon and remove its autostart entry before files are removed.
  ; User data (tasks.yaml, logs/) is preserved — only the service entry is cleared.
  ; ExecWait blocks until the daemon process exits (or times out per OS defaults).
  ExecWait '"$INSTDIR\labalaba-daemon.exe" cleanup'
!macroend
