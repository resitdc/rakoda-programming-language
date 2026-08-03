; ====================================================================
; RPL (Rakoda Programming Language) Installer - NSIS Script
; ====================================================================
; Menghasilkan: RPL-Setup-1.0.0.exe
; ====================================================================

!include "MUI2.nsh"
!include "EnvVarUpdate.nsh"

; ---- Metadata ----
!define PRODUCT_NAME "RPL"
!define PRODUCT_FULLNAME "Rakoda Programming Language"
!define PRODUCT_VERSION "1.0.0"
!define PRODUCT_PUBLISHER "Restu Dwi Cahyo"
!define PRODUCT_WEB_SITE "https://github.com/resitdc/rakoda-programming-language"
!define PRODUCT_DIR_REGKEY "Software\Microsoft\Windows\CurrentVersion\App Paths\rpl.exe"
!define PRODUCT_UNINST_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCT_NAME}"
!define INSTALL_DIR "$PROGRAMFILES\RPL"

; ---- General ----
Name "${PRODUCT_FULLNAME} ${PRODUCT_VERSION}"
OutFile "RPL-Setup-${PRODUCT_VERSION}.exe"
InstallDir "${INSTALL_DIR}"
InstallDirRegKey HKLM "${PRODUCT_DIR_REGKEY}" ""
RequestExecutionLevel admin
SetCompressor /SOLID lzma

; ---- UI Configuration ----
!define MUI_ABORTWARNING
!define MUI_ICON "..\..\installer\icon\rpl-icon.ico"
!define MUI_UNICON "..\..\installer\icon\rpl-icon.ico"
!define MUI_WELCOMEFINISHPAGE_BITMAP_NOSTRETCH
!define MUI_HEADERIMAGE

; ---- Pages ----
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "..\..\LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

; ---- Language ----
!insertmacro MUI_LANGUAGE "English"

; ====================================================================
; INSTALL SECTION
; ====================================================================
Section "RPL Runtime" SEC_MAIN
    SectionIn RO ; Required, cannot uncheck
    
    SetOutPath "$INSTDIR\bin"
    File "rpl.exe"
    
    SetOutPath "$INSTDIR"
    File "LICENSE"
    
    ; --- Add to System PATH ---
    ${EnvVarUpdate} $0 "PATH" "A" "HKLM" "$INSTDIR\bin"
    
    ; --- Create Uninstaller ---
    WriteUninstaller "$INSTDIR\uninstall.exe"
    
    ; --- Registry Entries ---
    WriteRegStr HKLM "${PRODUCT_DIR_REGKEY}" "" "$INSTDIR\bin\rpl.exe"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "DisplayName" "${PRODUCT_FULLNAME}"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "UninstallString" "$INSTDIR\uninstall.exe"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "DisplayIcon" "$INSTDIR\bin\rpl.exe"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "DisplayVersion" "${PRODUCT_VERSION}"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "Publisher" "${PRODUCT_PUBLISHER}"
    WriteRegStr HKLM "${PRODUCT_UNINST_KEY}" "URLInfoAbout" "${PRODUCT_WEB_SITE}"
    WriteRegDWORD HKLM "${PRODUCT_UNINST_KEY}" "NoModify" 1
    WriteRegDWORD HKLM "${PRODUCT_UNINST_KEY}" "NoRepair" 1
SectionEnd

; ====================================================================
; UNINSTALL SECTION
; ====================================================================
Section "Uninstall"
    ; --- Remove from PATH ---
    ${un.EnvVarUpdate} $0 "PATH" "R" "HKLM" "$INSTDIR\bin"
    
    ; --- Delete Files ---
    RMDir /r "$INSTDIR\bin"
    Delete "$INSTDIR\LICENSE"
    Delete "$INSTDIR\uninstall.exe"
    RMDir "$INSTDIR"
    
    ; --- Remove Registry Entries ---
    DeleteRegKey HKLM "${PRODUCT_UNINST_KEY}"
    DeleteRegKey HKLM "${PRODUCT_DIR_REGKEY}"
SectionEnd
