using System.Drawing.Drawing2D;
using System.Runtime.InteropServices;
using Microsoft.Win32;

namespace MouseHider;

/// <summary>
/// Paleta clara/escura seguindo o tema do Windows, mais os dois ajustes de janela do Win11
/// que dao o acabamento "atual": barra de titulo escura e cantos arredondados.
/// ponytail: sem Mica/Acrylic — WinForms pinta o BackColor por cima do backdrop e o resultado
/// so funciona com truques de TransparencyKey que quebram o clique. Cor chapada resolve.
/// </summary>
static class Theme
{
    public static bool Escuro { get; private set; } = LerTemaDoWindows();

    public static Color Fundo => Escuro ? Cor(0x20, 0x20, 0x20) : Cor(0xF9, 0xF9, 0xF9);
    public static Color Painel => Escuro ? Cor(0x2B, 0x2B, 0x2B) : Color.White;
    public static Color Texto => Escuro ? Cor(0xF2, 0xF2, 0xF2) : Cor(0x1A, 0x1A, 0x1A);
    public static Color TextoFraco => Escuro ? Cor(0x9A, 0x9A, 0x9A) : Cor(0x66, 0x66, 0x66);
    public static Color Borda => Escuro ? Cor(0x3D, 0x3D, 0x3D) : Cor(0xE1, 0xE1, 0xE1);
    public static Color Hover => Escuro ? Cor(0x38, 0x38, 0x38) : Cor(0xEF, 0xEF, 0xEF);
    public static Color Accent => Logo.Accent;

    static Color Cor(int r, int g, int b) => Color.FromArgb(r, g, b);

    static readonly string Familia = FonteDisponivel("Segoe UI Variable Text") ? "Segoe UI Variable Text" : "Segoe UI";
    public static Font Fonte(float tamanho = 9.5f, FontStyle estilo = FontStyle.Regular) => new(Familia, tamanho, estilo);

    static bool FonteDisponivel(string nome)
    {
        try { using var f = new FontFamily(nome); return true; }
        catch (ArgumentException) { return false; }
    }

    static bool LerTemaDoWindows()
    {
        var v = Registry.GetValue(
            @"HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
            "AppsUseLightTheme", 1);
        return v is int i && i == 0;
    }

    /// <summary>O usuario pode trocar o tema com a janela aberta; o SettingsForm reassina isso.</summary>
    public static void Reavaliar() => Escuro = LerTemaDoWindows();

    // --- acabamento de janela (Win11; em versoes antigas as chamadas apenas falham em silencio) ---

    const int DWMWA_USE_IMMERSIVE_DARK_MODE = 20;
    const int DWMWA_WINDOW_CORNER_PREFERENCE = 33;
    const int DWMWCP_ROUND = 2;

    [DllImport("dwmapi.dll")]
    static extern int DwmSetWindowAttribute(IntPtr hwnd, int attr, ref int valor, int tamanho);

    public static void AplicarNaJanela(Form f)
    {
        if (!f.IsHandleCreated) return;
        var escuro = Escuro ? 1 : 0;
        DwmSetWindowAttribute(f.Handle, DWMWA_USE_IMMERSIVE_DARK_MODE, ref escuro, sizeof(int));
        var canto = DWMWCP_ROUND;
        DwmSetWindowAttribute(f.Handle, DWMWA_WINDOW_CORNER_PREFERENCE, ref canto, sizeof(int));
    }

    /// <summary>Menu da bandeja no tema certo — o renderer padrao do WinForms e sempre claro.</summary>
    public sealed class MenuRenderer : ToolStripProfessionalRenderer
    {
        public MenuRenderer() : base(new Cores()) { RoundedEdges = true; }

        protected override void OnRenderItemText(ToolStripItemTextRenderEventArgs e)
        {
            e.TextColor = e.Item.Enabled ? Texto : TextoFraco;
            base.OnRenderItemText(e);
        }

        sealed class Cores : ProfessionalColorTable
        {
            public override Color MenuItemSelected => Hover;
            public override Color MenuItemSelectedGradientBegin => Hover;
            public override Color MenuItemSelectedGradientEnd => Hover;
            public override Color MenuItemBorder => Hover;
            public override Color MenuBorder => Borda;
            public override Color ToolStripDropDownBackground => Painel;
            public override Color ImageMarginGradientBegin => Painel;
            public override Color ImageMarginGradientMiddle => Painel;
            public override Color ImageMarginGradientEnd => Painel;
            public override Color SeparatorDark => Borda;
            public override Color SeparatorLight => Borda;
            public override Color CheckBackground => Accent;
            public override Color CheckSelectedBackground => Accent;
        }
    }

    public static Color Misturar(Color a, Color b, float t) => Color.FromArgb(
        (int)(a.R + (b.R - a.R) * t), (int)(a.G + (b.G - a.G) * t), (int)(a.B + (b.B - a.B) * t));
}
