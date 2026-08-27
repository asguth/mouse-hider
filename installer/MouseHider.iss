; Instalador do Mouse Hider — Inno Setup 6
; Compilar:  ISCC.exe /DMyAppVersion=1.0.0 installer\MouseHider.iss
; O nome do arquivo de saida NUNCA muda: e ele que sustenta o link permanente
; https://github.com/asguth/mouse-hider/releases/latest/download/MouseHiderSetup.exe

#ifndef MyAppVersion
  #define MyAppVersion "1.0.0"
#endif

#define MyAppName "Mouse Hider"
#define MyAppExe "MouseHider.exe"
#define MyAppUrl "https://github.com/asguth/mouse-hider"

[Setup]
; AppId identifica o programa entre versoes — nao trocar nunca, senao o Windows
; passa a tratar cada versao como um app diferente e o desinstalador se perde.
AppId={{8F3C1A64-9D2B-4E7A-B5C1-2A6D4F8E0C37}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher=asguth
AppPublisherURL={#MyAppUrl}
AppSupportURL={#MyAppUrl}/issues
AppUpdatesURL={#MyAppUrl}/releases

; Caminhos do script sao relativos a raiz do repositorio, nao a pasta installer\.
SourceDir=..

; lowest = instala so para o usuario atual, sem prompt de administrador.
PrivilegesRequired=lowest
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
UninstallDisplayIcon={app}\{#MyAppExe}
OutputDir=installer\Output
OutputBaseFilename=MouseHiderSetup
SetupIconFile=MouseHider\assets\MouseHider.ico
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
; Fecha o app pelo Restart Manager antes de sobrescrever o exe.
CloseApplications=yes
RestartApplications=no

[Languages]
Name: "brazilianportuguese"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; A pasta inteira: o app e publicado self-contained sem arquivo unico, para nao parecer
; packer para heuristica de antivirus. O instalador e que entrega tudo junto.
Source: "dist\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExe}"; Tasks: desktopicon

[Run]
; Sem skipifsilent: na instalacao silenciosa da auto-atualizacao o app precisa
; voltar sozinho, senao ele so sumiria da bandeja. No modo interativo continua
; sendo a caixinha "executar agora" do fim do assistente.
Filename: "{app}\{#MyAppExe}"; Description: "{cm:LaunchProgram,{#MyAppName}}"; Flags: nowait postinstall

[Code]
// O autostart e gravado pelo proprio app em HKCU\...\Run. Se ficar para tras depois
// de desinstalar, o Windows tenta abrir um exe que nao existe mais a cada boot.
procedure CurUninstallStepChanged(CurStep: TUninstallStep);
begin
  if CurStep = usPostUninstall then
    RegDeleteValue(HKEY_CURRENT_USER,
      'Software\Microsoft\Windows\CurrentVersion\Run', 'MouseHider');
end;
