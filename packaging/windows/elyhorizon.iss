; Elyhorizon distribution installer. Compile only the explicitly staged programs.
#ifndef AppVersion
  #define AppVersion "0.2.0"
#endif
#ifndef NumericVersion
  #define NumericVersion "0.2.0.0"
#endif
#ifndef InstallerAppId
  #define InstallerAppId "{{7DA81F8A-49B8-46B2-A651-2DEB33B761D2}"
#endif
#ifndef PayloadDir
  #error PayloadDir must name the staged Elyhorizon folder
#endif
#ifndef BuildOutputDir
  #error BuildOutputDir must name the installer output folder
#endif

[Setup]
AppId={#InstallerAppId}
AppName=Elyhorizon
AppVersion={#AppVersion}
AppPublisher=Elyhorizon
AppPublisherURL=https://github.com/kkerneldev/Elyhorizon
AppSupportURL=https://github.com/kkerneldev/Elyhorizon/issues
AppUpdatesURL=https://github.com/kkerneldev/Elyhorizon/releases
DefaultDirName={localappdata}\Programs\Elyhorizon
DefaultGroupName=Elyhorizon
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
OutputDir={#BuildOutputDir}
OutputBaseFilename=Elyhorizon-{#AppVersion}-windows-x64-setup
SetupIconFile=..\icons\horizon.ico
UninstallDisplayIcon={app}\horizon-launcher.exe
VersionInfoVersion={#NumericVersion}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
LicenseFile={#PayloadDir}\LICENSE
CloseApplications=yes
RestartApplications=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#PayloadDir}\horizon-launcher.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\horizon-client.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\LICENSE"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\THIRD_PARTY_NOTICES.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\elyby-account.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#PayloadDir}\README.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Elyhorizon"; Filename: "{app}\horizon-launcher.exe"; WorkingDir: "{app}"
Name: "{autodesktop}\Elyhorizon"; Filename: "{app}\horizon-launcher.exe"; WorkingDir: "{app}"; Tasks: desktopicon

[Run]
Filename: "{app}\horizon-launcher.exe"; Description: "{cm:LaunchProgram,Elyhorizon}"; Flags: nowait postinstall skipifsilent
