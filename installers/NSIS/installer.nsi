;NSIS Modern User Interface
;TidaLuna Installer Script

;--------------------------------
;Imports
	

	!include "MUI2.nsh"
	!include "FileFunc.nsh"
	!include "LogicLib.nsh"
	!include "headers.nsh"


;--------------------------------
;General

	;Name and file
	Name "${PRODUCT_NAME} ${PRODUCT_VERSION}"
	OutFile "TidaLuna Installer.exe"
	Icon "${ASSETS_ROOT}\icon.ico"
	Unicode True
	BrandingText "TidaLuna Installer"

	;Default installation folder
	InstallDir "$LOCALAPPDATA\TidaLunaLauncher"

	;Get installation folder from registry if available
	InstallDirRegKey HKCU "Software\TidaLuna Launcher" ""

	ShowInstDetails show
	ShowUnInstDetails show

	;Request application privileges for Windows Vista
	RequestExecutionLevel user

;--------------------------------
;Interface Settings

	!define MUI_ABORTWARNING
	
	!define MUI_ICON "${ASSETS_ROOT}\icon.ico"
	!define MUI_UNICON "${ASSETS_ROOT}\icon.ico"
	!define MUI_UI_HEADERIMAGE_RIGHT "${ASSETS_ROOT}\icon.bmp"

	!define MUI_WELCOMEFINISHPAGE_BITMAP "${ASSETS_ROOT}\welcome.bmp"
	!define MUI_WELCOMEPAGE_TEXT "Welcome to the TidaLuna Installer.$\n\
	$\n\
	This will install TidaLuna, a client mod for the TIDAL music app.$\n\
	$\n\
	Make sure TIDAL is closed before proceeding."

	!define MUI_COMPONENTSPAGE_SMALLDESC

	!define MUI_FINISHPAGE_NOAUTOCLOSE
	!define MUI_UNFINISHPAGE_NOAUTOCLOSE

;--------------------------------
;Pages

	 !insertmacro MUI_PAGE_WELCOME
;	 !insertmacro MUI_PAGE_LICENSE "${NSISDIR}\Docs\Modern UI\License.txt"
	!define MUI_PAGE_CUSTOMFUNCTION_LEAVE InstallLeave
	!insertmacro MUI_PAGE_COMPONENTS
	!insertmacro MUI_PAGE_DIRECTORY
	!insertmacro MUI_PAGE_INSTFILES
	!insertmacro MUI_PAGE_FINISH

	!insertmacro MUI_UNPAGE_WELCOME
	!insertmacro MUI_UNPAGE_CONFIRM
	!insertmacro MUI_UNPAGE_INSTFILES
	!insertmacro MUI_UNPAGE_FINISH

;--------------------------------
;Languages

	!insertmacro MUI_LANGUAGE "English"

;--------------------------------
;Installer Attributes

	VIProductVersion "${PRODUCT_VERSION}"
	VIAddVersionKey /LANG=${LANG_ENGLISH} "ProductName" "${PRODUCT_NAME}"
	VIAddVersionKey /LANG=${LANG_ENGLISH} "CompanyName" "${PRODUCT_PUBLISHER}"
	VIAddVersionKey /LANG=${LANG_ENGLISH} "FileVersion" "${PRODUCT_VERSION}"
	VIAddVersionKey /LANG=${LANG_ENGLISH} "ProductVersion" "${PRODUCT_VERSION}"
	VIAddVersionKey /LANG=${LANG_ENGLISH} "FileDescription" "${PRODUCT_NAME} Installer"
	VIAddVersionKey /LANG=${LANG_ENGLISH} "LegalCopyright" "© ${PRODUCT_PUBLISHER}"

;--------------------------------
;Installer Sections


	!include "sections\install.nsi"

	!include "sections\uninstall.nsi"
	
Function .onInit

	; TidaLuna install is always selected by default
	SectionSetFlags ${InstallTidaLuna} ${SF_SELECTED}

FunctionEnd

;--------------------------------
;Descriptions

	;Language strings
	LangString DESC_InstallTidaLuna ${LANG_ENGLISH} "Install TidaLuna launcher for TIDAL"
	LangString DESC_StartMenuShortcuts ${LANG_ENGLISH} "Create Start Menu shortcuts"

	;Assign language strings to sections
	!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
		!insertmacro MUI_DESCRIPTION_TEXT ${InstallTidaLuna} $(DESC_InstallTidaLuna)
		!insertmacro MUI_DESCRIPTION_TEXT ${StartMenuShortcuts} $(DESC_StartMenuShortcuts)
	!insertmacro MUI_FUNCTION_DESCRIPTION_END


	LangString DESC_UninstallTidaLuna ${LANG_ENGLISH} "Uninstall TidaLuna"

	!insertmacro MUI_UNFUNCTION_DESCRIPTION_BEGIN
		!insertmacro MUI_DESCRIPTION_TEXT ${UninstallTidaLuna} $(DESC_UninstallTidaLuna)
	!insertmacro MUI_UNFUNCTION_DESCRIPTION_END


!addplugindir "plugins"

!define FindProc_NOT_FOUND 1
!define FindProc_FOUND 0

!macro check_running_tidal un
Function ${un}CheckRunningTidal
	
	NsProcessW::_FindProcess "TIDAL.exe"
	Pop $R0

	${If} $R0 == 0
		MessageBox MB_OKCANCEL|MB_ICONEXCLAMATION "TIDAL is running. Click OK to terminate it." /SD IDOK IDCANCEL +2
		NsProcessW::_KillProcess "TIDAL.exe"
		Abort
	${EndIf}

FunctionEnd
!macroend

!insertmacro check_running_tidal ""
!insertmacro check_running_tidal "un."

Function InstallLeave
	Call CheckRunningTidal
FunctionEnd
