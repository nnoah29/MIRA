; Installeur Windows de MIRA (Inno Setup 6). Paramètres : /DVersion=X.Y.Z /DStage=<dossier à installer> /DOut=<dossier de sortie>.
#ifndef Version
  #define Version "0.0.0"
#endif
#ifndef Stage
  #error "Stage : dossier à installer (voir build-installer.ps1)"
#endif
#ifndef Out
  #define Out "out"
#endif
#define AppId "com.toffanoah.mira"

[Setup]
AppId={{8F3C1B7A-52D4-4E9B-A6C1-3D7E0B9F4A21}
AppName=MIRA
AppVersion={#Version}
AppPublisher=toffanoah
AppPublisherURL=https://toffanoah.com/en/mira
AppSupportURL=https://github.com/nnoah29/MIRA/issues
AppUpdatesURL=https://github.com/nnoah29/MIRA/releases
DefaultDirName={autopf}\MIRA
DefaultGroupName=MIRA
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir={#Out}
OutputBaseFilename=mira-mail-{#Version}-windows-x64-setup
SetupIconFile=..\..\app\windows\runner\resources\app_icon.ico
UninstallDisplayIcon={app}\mira.exe
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
DisableProgramGroupPage=yes
LicenseFile=..\..\LICENSE

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "french"; MessagesFile: "compiler:Languages\French.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; Flags: unchecked

[Files]
Source: "{#Stage}\*"; DestDir: "{app}"; Flags: recursesubdirs ignoreversion

[Icons]
Name: "{autoprograms}\MIRA"; Filename: "{app}\mira.exe"; AppUserModelID: "{#AppId}"
Name: "{autodesktop}\MIRA"; Filename: "{app}\mira.exe"; AppUserModelID: "{#AppId}"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueName: "MIRA"; ValueType: none; Flags: dontcreatekey uninsdeletevalue
Root: HKCU; Subkey: "Software\Classes\AppUserModelId\{#AppId}"; ValueType: none; Flags: dontcreatekey uninsdeletekey

[Run]
Filename: "{app}\mira.exe"; Description: "{cm:LaunchProgram,MIRA}"; Flags: nowait postinstall skipifsilent

[Code]
procedure StopMira();
var
  Code: Integer;
begin
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/F /IM mira-daemon.exe', '', SW_HIDE, ewWaitUntilTerminated, Code);
  Exec(ExpandConstant('{sys}\taskkill.exe'), '/F /IM mira.exe', '', SW_HIDE, ewWaitUntilTerminated, Code);
end;

function PrepareToInstall(var NeedsRestart: Boolean): String;
begin
  StopMira();
  Result := '';
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usUninstall then
    StopMira();
end;
