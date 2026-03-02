Section "un.TidaLuna" UninstallTidaLuna

  Delete "$INSTDIR\TidaLuna.exe"
  RMDir "$INSTDIR"

  Delete "$SMPROGRAMS\TidaLuna\TidaLuna.lnk"
  RMDir "$SMPROGRAMS\TidaLuna"

  DeleteRegKey HKCU "Software\TidaLuna Launcher"
  DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\TidaLuna"

SectionEnd

Function un.onUninstSuccess
  ; Clean up cache folder and uninstaller
  RMDir /r "$INSTDIR\cache"
  Delete "$INSTDIR\Uninstall TidaLuna.exe"
  RMDir "$INSTDIR"
FunctionEnd
