Section "Create Start Menu Shortcuts" StartMenuShortcuts
	; This section is selected by default and acts as a flag.
	; The actual shortcut creation is done in the install section below.
SectionEnd

Section "TIDAL (TidaLuna)" InstallTidaLuna

	SetOutPath "$INSTDIR"
	File "/oname=TidaLuna.exe" "${BINARIES_ROOT}\tidaluna.exe"

	WriteRegStr HKCU "Software\TidaLuna Launcher" "" "$INSTDIR"

	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\TidaLuna" "DisplayName" "TidaLuna"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\TidaLuna" "HelpLink" "https://github.com/Inrixia/TidaLuna"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\TidaLuna" "InstallLocation" "$INSTDIR"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\TidaLuna" "InstallSource" "https://github.com/Inrixia/TidaLuna"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\TidaLuna" "UninstallString" "$\"$INSTDIR\Uninstall TidaLuna.exe$\""
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\TidaLuna" "QuietUninstallString" "$\"$INSTDIR\Uninstall TidaLuna.exe$\" /S"
	WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\TidaLuna" "DisplayIcon" "$INSTDIR\TidaLuna.exe"

	; Only create shortcut if the Start Menu section is selected
	SectionGetFlags ${StartMenuShortcuts} $0
	${If} $0 & ${SF_SELECTED}
		; Only create shortcut if it doesn't already exist
		IfFileExists "$SMPROGRAMS\TidaLuna\TidaLuna.lnk" +3 0
			CreateDirectory "$SMPROGRAMS\TidaLuna"
			CreateShortCut "$SMPROGRAMS\TidaLuna\TidaLuna.lnk" "$INSTDIR\TidaLuna.exe" "" "$INSTDIR\TidaLuna.exe"
	${EndIf}

SectionEnd

Function .onInstSuccess

	WriteUninstaller "$INSTDIR\Uninstall TidaLuna.exe"

FunctionEnd
