using System.Text.Json;
using System.Text.Json.Serialization;
using Microsoft.Win32;

namespace MouseHider;

sealed class Config
{
    public int IdleSeconds { get; set; } = 5;
    public bool Paused { get; set; }
    public string Hotkey { get; set; } = "Ctrl+Alt+H";
    public string Idioma { get; set; } = "auto";

    static readonly string Dir = Path.Combine(
        Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData), "MouseHider");
    static readonly string ConfigPath = Path.Combine(Dir, "config.json");
    static readonly string SentinelPath = Path.Combine(Dir, "cursor-hidden.flag");
    static readonly string LogPath = Path.Combine(Dir, "log.txt");

    const string RunKey = @"Software\Microsoft\Windows\CurrentVersion\Run";
    const string RunValue = "MouseHider";
    const long LogMaxBytes = 256 * 1024;

    public static string Folder => Dir;

    /// <summary>Primeira execucao: nao ha config gravada ainda.</summary>
    public static bool PrimeiraVez => !File.Exists(ConfigPath);

    public static Config Load()
    {
        // ponytail: config ilegivel/corrompida cai no default em silencio. Nao vale uma tela de erro
        // num app que roda invisivel — o motivo fica no log.
        try
        {
            return JsonSerializer.Deserialize<Config>(File.ReadAllText(ConfigPath)) ?? new Config();
        }
        catch (FileNotFoundException) { return new Config(); }
        catch (DirectoryNotFoundException) { return new Config(); }
        catch (Exception e) { Log("config ilegivel, usando padrao: " + e.Message); return new Config(); }
    }

    public void Save()
    {
        try
        {
            Directory.CreateDirectory(Dir);
            File.WriteAllText(ConfigPath, JsonSerializer.Serialize(this, JsonOpts));
        }
        catch (Exception e) { Log("falha ao salvar config: " + e.Message); }
    }

    // Encoder relaxado: sem ele o "+" do atalho vira "+" e o arquivo fica ruim de editar na mao.
    static readonly JsonSerializerOptions JsonOpts = new()
    {
        WriteIndented = true,
        Encoder = System.Text.Encodings.Web.JavaScriptEncoder.UnsafeRelaxedJsonEscaping
    };

    // --- sentinela de crash: existe enquanto o cursor esta escondido ---

    public static bool SentinelExists => File.Exists(SentinelPath);

    public static void SetSentinel(bool on)
    {
        try
        {
            if (on) { Directory.CreateDirectory(Dir); File.WriteAllText(SentinelPath, DateTime.Now.ToString("s")); }
            else if (File.Exists(SentinelPath)) File.Delete(SentinelPath);
        }
        catch (Exception e) { Log("falha na sentinela: " + e.Message); }
    }

    // --- autostart ---

    public static bool AutoStart
    {
        get
        {
            using var k = Registry.CurrentUser.OpenSubKey(RunKey);
            return k?.GetValue(RunValue) != null;
        }
        set
        {
            try
            {
                using var k = Registry.CurrentUser.OpenSubKey(RunKey, writable: true);
                if (k == null) return;
                if (value) k.SetValue(RunValue, "\"" + Environment.ProcessPath + "\"");
                else k.DeleteValue(RunValue, throwOnMissingValue: false);
            }
            catch (Exception e) { Log("falha no autostart: " + e.Message); }
        }
    }

    /// <summary>
    /// Se o autostart esta ligado mas aponta para outro caminho, reescreve para o exe atual.
    /// Sem isso, mover ou reinstalar o app deixa o Windows tentando abrir um arquivo que nao
    /// existe mais — e falha calada, ninguem descobre ate perceber que nao inicia sozinho.
    /// </summary>
    public static void CorrigirAutoStart()
    {
        try
        {
            using var k = Registry.CurrentUser.OpenSubKey(RunKey, writable: true);
            if (k?.GetValue(RunValue) is not string atual) return;

            var esperado = "\"" + Environment.ProcessPath + "\"";
            if (string.Equals(atual, esperado, StringComparison.OrdinalIgnoreCase)) return;

            k.SetValue(RunValue, esperado);
            Log("autostart corrigido: " + atual + " -> " + esperado);
        }
        catch (Exception e) { Log("falha ao corrigir autostart: " + e.Message); }
    }

    // --- log ---

    public static void Log(string msg)
    {
        try
        {
            Directory.CreateDirectory(Dir);
            // ponytail: "rotacao" = apagar ao passar de 256 KB. Log de diagnostico, nao de auditoria.
            var fi = new FileInfo(LogPath);
            if (fi.Exists && fi.Length > LogMaxBytes) File.Delete(LogPath);
            // UTF-8 com BOM: sem ele o Bloco de Notas e o PowerShell leem acento como ANSI e mostram "â€".
            File.AppendAllText(LogPath, DateTime.Now.ToString("yyyy-MM-dd HH:mm:ss") + "  " + msg + Environment.NewLine,
                new System.Text.UTF8Encoding(encoderShouldEmitUTF8Identifier: true));
        }
        catch { /* log nunca derruba o app */ }
    }
}
