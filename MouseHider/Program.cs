using System.Runtime.InteropServices;
using Microsoft.Win32;

namespace MouseHider;

static class Program
{
    static Mutex? _mutex;
    static Config _config = null!;
    static NotifyIcon _tray = null!;
    static ToolStripMenuItem _pauseItem = null!;
    static ToolStripMenuItem _configItem = null!;
    static ToolStripMenuItem _sairItem = null!;
    static SettingsForm? _settings;
    static HotkeyWindow? _hotkey;

    [STAThread]
    static int Main(string[] args)
    {
        if (args.Contains("--selftest")) return SelfTest.Run();

        // Gera assets/MouseHider.ico (o icone do .exe). Roda de novo so quando o logo mudar.
        if (args.Contains("--makeicon"))
        {
            var destino = Path.Combine(AppContext.BaseDirectory, "..", "..", "..", "assets", "MouseHider.ico");
            Logo.WriteIco(Path.GetFullPath(destino));
            return 0;
        }

        // Duas instancias brigando pelo estado do cursor = cursor preso invisivel.
        _mutex = new Mutex(true, @"Local\MouseHider", out var primeira);
        if (!primeira) return 0;

        ApplicationConfiguration.Initialize();

        // Antes de tudo: se a instancia anterior morreu escondida, devolve o cursor.
        CursorHider.RestoreIfStale();
        var primeiraVez = Config.PrimeiraVez;
        _config = Config.Load();
        Idiomas.Definir(_config.Idioma);
        Config.CorrigirAutoStart();

        // Todo caminho de saida restaura o cursor. Este e o requisito critico do app.
        AppDomain.CurrentDomain.ProcessExit += (_, _) => CursorHider.Restore();
        AppDomain.CurrentDomain.UnhandledException += (_, e) =>
        {
            Config.Log("crash: " + e.ExceptionObject);
            CursorHider.Restore();
        };
        Application.ThreadException += (_, e) =>
        {
            Config.Log("erro na UI: " + e.Exception);
            CursorHider.Restore();
        };
        Application.ApplicationExit += (_, _) => CursorHider.Restore();
        SystemEvents.SessionEnding += (_, _) => CursorHider.Restore();
        SystemEvents.SessionSwitch += (_, _) => CursorHider.Restore();

        // Claro/escuro pode mudar com o app aberto; o menu le a paleta na hora de pintar.
        SystemEvents.UserPreferenceChanged += (_, _) => Theme.Reavaliar();

        _pauseItem = new ToolStripMenuItem("", null, (_, _) => TogglePause()) { Checked = _config.Paused };
        _configItem = new ToolStripMenuItem("", null, (_, _) => ShowSettings());
        _sairItem = new ToolStripMenuItem("", null, (_, _) => Application.Exit());
        var menu = new ContextMenuStrip
        {
            Renderer = new Theme.MenuRenderer(),
            Font = Theme.Fonte(),
            BackColor = Theme.Painel,
            ForeColor = Theme.Texto,
            ShowImageMargin = false
        };
        menu.Items.Add(_configItem);
        menu.Items.Add(_pauseItem);
        menu.Items.Add(new ToolStripSeparator());
        menu.Items.Add(_sairItem);

        _tray = new NotifyIcon
        {
            Icon = Logo.Tray(_config.Paused),
            Text = TextoDaBandeja(),
            Visible = true,
            ContextMenuStrip = menu
        };
        _tray.DoubleClick += (_, _) => ShowSettings();
        AtualizarTextos();

        var timer = new System.Windows.Forms.Timer { Interval = 500 };
        timer.Tick += (_, _) => Tick();
        timer.Start();

        // Primeira execucao abre a janela: sem isso o app some na bandeja e parece que nao instalou.
        if (primeiraVez) ShowSettings();

        _hotkey = new HotkeyWindow(() => TogglePause(viaAtalho: true));
        AplicarAtalho(_config.Hotkey);

        Config.Log("iniciado (idle=" + _config.IdleSeconds + "s, pausado=" + _config.Paused +
                   ", atalho=" + _config.Hotkey + ", idioma=" + _config.Idioma + ")");
        Application.Run();

        _hotkey.Dispose();
        _tray.Visible = false;
        CursorHider.Restore();
        return 0;
    }

    /// <summary>Registra o atalho global. Devolve false se a combinacao ja estiver tomada por outro app.</summary>
    public static bool AplicarAtalho(string atalho)
    {
        if (_hotkey is null) return false;
        var ok = _hotkey.Registrar(atalho);
        Config.Log(ok ? "atalho registrado: " + atalho : "atalho recusado (em uso por outro app): " + atalho);
        return ok;
    }

    /// <summary>Reaplica os textos do menu apos troca de idioma.</summary>
    public static void AtualizarTextos()
    {
        _configItem.Text = Idiomas.T("menu_config");
        _pauseItem.Text = Idiomas.T("menu_pausar");
        _sairItem.Text = Idiomas.T("menu_sair");
        _tray.Text = TextoDaBandeja();
    }

    static void Tick()
    {
        var limiar = (uint)_config.IdleSeconds * 1000u;
        if (!_config.Paused && Native.IdleMilliseconds() >= limiar) CursorHider.Hide();
        else CursorHider.Restore();
    }

    static void TogglePause(bool viaAtalho = false)
    {
        _config.Paused = !_config.Paused;
        _pauseItem.Checked = _config.Paused;
        _tray.Icon = Logo.Tray(_config.Paused); // pausado = logo em cinza
        _tray.Text = TextoDaBandeja();
        if (_config.Paused) CursorHider.Restore();
        _config.Save();
        Config.Log(_config.Paused ? "pausado" : "retomado");

        // Pelo menu da bandeja o proprio check ja da o retorno; pelo atalho global nao ha nada
        // na tela, entao um balao e a unica forma do usuario saber que funcionou.
        if (viaAtalho)
            _tray.ShowBalloonTip(1200, "Mouse Hider",
                Idiomas.T(_config.Paused ? "pausado" : "ativo"), ToolTipIcon.None);
    }

    static string TextoDaBandeja() =>
        _config.Paused ? "Mouse Hider — " + Idiomas.T("pausado") : "Mouse Hider";

    static void ShowSettings()
    {
        if (_settings is null || _settings.IsDisposed) _settings = new SettingsForm(_config);
        // Estado antes do Show(): se a janela tinha sido minimizada por fora, nao pisca restaurando.
        _settings.WindowState = FormWindowState.Normal;
        _settings.Show();
        _settings.BringToFront();
        _settings.Activate();
    }
}

static class Native
{
    [StructLayout(LayoutKind.Sequential)]
    struct LASTINPUTINFO
    {
        public uint cbSize;
        public uint dwTime;
    }

    [DllImport("user32.dll")]
    static extern bool GetLastInputInfo(ref LASTINPUTINFO plii);

    /// <summary>Tempo desde a ultima entrada de mouse/teclado, em ms. Nao exige hook global.</summary>
    public static uint IdleMilliseconds()
    {
        var lii = new LASTINPUTINFO { cbSize = (uint)Marshal.SizeOf<LASTINPUTINFO>() };
        if (!GetLastInputInfo(ref lii)) return 0;
        return IdleFrom(unchecked((uint)Environment.TickCount), lii.dwTime);
    }

    /// <summary>Subtracao unchecked em uint: cobre o wraparound do TickCount a cada 49,7 dias.</summary>
    public static uint IdleFrom(uint agora, uint ultima) => unchecked(agora - ultima);
}

/// <summary>Janela oculta so para receber WM_HOTKEY do atalho global.</summary>
sealed class HotkeyWindow : NativeWindow, IDisposable
{
    const int WM_HOTKEY = 0x0312;
    const int HotkeyId = 1;

    [DllImport("user32.dll")]
    static extern bool RegisterHotKey(IntPtr hWnd, int id, int fsModifiers, int vk);

    [DllImport("user32.dll")]
    static extern bool UnregisterHotKey(IntPtr hWnd, int id);

    readonly Action _onHotkey;
    bool _registrado;

    public HotkeyWindow(Action onHotkey)
    {
        _onHotkey = onHotkey;
        CreateHandle(new CreateParams());
    }

    /// <summary>Troca o atalho ativo. Se a combinacao nova falhar, fica sem atalho e devolve false.</summary>
    public bool Registrar(string atalho)
    {
        if (_registrado)
        {
            UnregisterHotKey(Handle, HotkeyId);
            _registrado = false;
        }

        var combo = Atalho.Interpretar(atalho);
        if (combo is null) return false;

        _registrado = RegisterHotKey(Handle, HotkeyId, combo.Value.Mods, combo.Value.Vk);
        return _registrado;
    }

    protected override void WndProc(ref Message m)
    {
        if (m.Msg == WM_HOTKEY) _onHotkey();
        base.WndProc(ref m);
    }

    public void Dispose()
    {
        if (_registrado) UnregisterHotKey(Handle, HotkeyId);
        DestroyHandle();
    }
}

/// <summary>MouseHider.exe --selftest: valida a aritmetica de idle e o ciclo hide/restore. Sai com 0 ou 1.</summary>
static class SelfTest
{
    [DllImport("kernel32.dll")]
    static extern bool AttachConsole(int pid);

    public static int Run()
    {
        AttachConsole(-1); // ATTACH_PARENT_PROCESS: WinExe nao tem console propria
        try
        {
            Check(Native.IdleFrom(5000, 1000) == 4000, "idle simples");
            Check(Native.IdleFrom(unchecked((uint)int.MinValue), unchecked((uint)int.MaxValue)) == 1,
                  "wraparound do TickCount");

            CursorHider.Hide();
            Check(CursorHider.IsHidden, "estado escondido apos Hide");
            Check(Config.SentinelExists, "sentinela criada ao esconder");

            CursorHider.Restore();
            Check(!CursorHider.IsHidden, "estado visivel apos Restore");
            Check(!Config.SentinelExists, "sentinela removida ao restaurar");

            CursorHider.Restore(); // idempotente
            Console.WriteLine("selftest OK");
            return 0;
        }
        catch (Exception e)
        {
            CursorHider.Restore(); // nunca deixar o cursor escondido por causa do teste
            Console.Error.WriteLine("selftest FALHOU: " + e.Message);
            return 1;
        }
    }

    static void Check(bool ok, string oque)
    {
        if (!ok) throw new Exception(oque);
    }
}
