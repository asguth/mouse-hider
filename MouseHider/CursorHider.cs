using System.Runtime.InteropServices;

namespace MouseHider;

/// <summary>
/// Esconde e restaura os cursores do sistema inteiro (vale para todos os processos).
/// ShowCursor() nao serve aqui: e per-thread e nao afeta outras janelas.
/// </summary>
static class CursorHider
{
    const uint SPI_SETCURSORS = 0x0057;
    const uint SPIF_SENDCHANGE = 0x02;

    // ponytail: lista fixa dos OCR_* padrao do Windows. Cursores proprios de apps individuais
    // continuam visiveis — nao ha API publica para enumerar cursores de terceiros.
    static readonly int[] CursorIds =
    {
        32512,                             // OCR_NORMAL
        32513, 32514, 32515, 32516,        // IBEAM, WAIT, CROSS, UP
        32642, 32643, 32644, 32645, 32646, // SIZENWSE, SIZENESW, SIZEWE, SIZENS, SIZEALL
        32648, 32649, 32650                // NO, HAND, APPSTARTING
    };

    [DllImport("user32.dll", SetLastError = true)]
    static extern IntPtr CreateCursor(IntPtr hInst, int xHotSpot, int yHotSpot,
        int nWidth, int nHeight, byte[] pvANDPlane, byte[] pvXORPlane);

    [DllImport("user32.dll", SetLastError = true)]
    static extern bool SetSystemCursor(IntPtr hcur, int id);

    [DllImport("user32.dll", SetLastError = true)]
    static extern bool SystemParametersInfo(uint uiAction, uint uiParam, IntPtr pvParam, uint fWinIni);

    static readonly object Gate = new();
    static bool _hidden;

    public static bool IsHidden { get { lock (Gate) return _hidden; } }

    public static void Hide()
    {
        lock (Gate)
        {
            if (_hidden) return;

            // AND=1 + XOR=0 em cada pixel => cursor totalmente transparente.
            var and = new byte[32 * 32 / 8];
            Array.Fill(and, (byte)0xFF);
            var xor = new byte[32 * 32 / 8];

            var trocados = 0;
            foreach (var id in CursorIds)
            {
                // Um handle novo por id: SetSystemCursor assume a posse e destroi o handle.
                var h = CreateCursor(IntPtr.Zero, 0, 0, 32, 32, and, xor);
                if (h == IntPtr.Zero) continue;
                if (SetSystemCursor(h, id)) trocados++;
            }

            if (trocados == 0)
            {
                Config.Log("nenhum cursor pode ser trocado — desistindo do hide");
                return;
            }

            _hidden = true;
            Config.SetSentinel(true);
        }
    }

    /// <summary>Recarrega os cursores padrao do registry. Idempotente e seguro de chamar de qualquer lugar.</summary>
    public static void Restore()
    {
        lock (Gate)
        {
            if (!_hidden && !Config.SentinelExists) return;
            SystemParametersInfo(SPI_SETCURSORS, 0, IntPtr.Zero, SPIF_SENDCHANGE);
            _hidden = false;
            Config.SetSentinel(false);
        }
    }

    /// <summary>Instancia anterior morreu com o cursor escondido: restaura antes de qualquer outra coisa.</summary>
    public static void RestoreIfStale()
    {
        if (!Config.SentinelExists) return;
        Config.Log("sentinela encontrada no startup — restaurando cursor da sessao anterior");
        Restore();
    }
}
