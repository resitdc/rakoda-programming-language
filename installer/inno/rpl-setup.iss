; ====================================================================
; RPL (Rakoda Programming Language) Installer - Inno Setup Script
; ====================================================================
; Menghasilkan: RPL-Setup-1.0.0-x64.exe / RPL-Setup-1.0.0-x86.exe
; ====================================================================

#define MyAppName "RPL"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "Restu Dwi Cahyo"
#define MyAppURL "https://github.com/resitdc/rakoda-programming-language"
#define MyAppExeName "rpl.exe"

[Setup]
AppId={{C7892305-6490-4822-BDD5-5028919A0A8E}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
LicenseFile=LICENSE
OutputDir=.
OutputBaseFilename=RPL-Setup-{#MyAppVersion}-{#AppArch}
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ChangesEnvironment=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
Source: "rpl.exe"; DestDir: "{app}\bin"; Flags: ignoreversion
Source: "examples\*"; DestDir: "{app}\examples"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "documentation\*"; DestDir: "{app}\docs"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "LICENSE"; DestDir: "{app}"; Flags: ignoreversion

[Registry]
; Tambahkan Registry Editor Key untuk Windows App Paths
Root: HKLM; Subkey: "Software\Microsoft\Windows\CurrentVersion\App Paths\rpl.exe"; ValueType: string; ValueName: ""; ValueData: "{app}\bin\rpl.exe"; Flags: uninsdeletekey
Root: HKLM; Subkey: "Software\Microsoft\Windows\CurrentVersion\App Paths\rpl.exe"; ValueType: string; ValueName: "Path"; ValueData: "{app}\bin"; Flags: uninsdeletekey

[Code]
const
  EnvironmentKey = 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment';

procedure AddToPath();
var
  OldPath, NewPath: string;
begin
  if RegQueryStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', OldPath) then
  begin
    if Pos(';' + ExpandConstant('{app}\bin'), ';' + OldPath) = 0 then
    begin
      NewPath := OldPath + ';' + ExpandConstant('{app}\bin');
      RegWriteExpandStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', NewPath);
    end;
  end;
end;

procedure RemoveFromPath();
var
  OldPath, NewPath: string;
  PathToFind: string;
  P: Integer;
begin
  if RegQueryStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', OldPath) then
  begin
    PathToFind := ';' + ExpandConstant('{app}\bin');
    P := Pos(PathToFind, OldPath);
    if P > 0 then
    begin
      NewPath := Copy(OldPath, 1, P - 1) + Copy(OldPath, P + Length(PathToFind), Length(OldPath));
      RegWriteExpandStringValue(HKEY_LOCAL_MACHINE, EnvironmentKey, 'Path', NewPath);
    end;
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
  begin
    AddToPath();
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    RemoveFromPath();
  end;
end;
