using System.Drawing.Drawing2D;
using System.Drawing.Imaging;

namespace MouseHider;

/// <summary>
/// O logo desenhado por codigo: cursor branco que se dissolve sobre um losango arredondado.
/// ponytail: nada de asset binario para manter — o .ico e gerado com --makeicon e a bandeja
/// renderiza direto em memoria, entao mudar a marca e mudar duas cores aqui.
/// </summary>
static class Logo
{
    static readonly Color Inicio = Color.FromArgb(0x63, 0x66, 0xF1); // indigo
    static readonly Color Fim = Color.FromArgb(0x8B, 0x5C, 0xF6);    // violeta
    static readonly Color InicioOff = Color.FromArgb(0x6B, 0x70, 0x80);
    static readonly Color FimOff = Color.FromArgb(0x9C, 0xA3, 0xAF);

    public static Color Accent => Inicio;

    // Contorno do cursor num espaco 12.2 x 20, normalizado na hora de desenhar.
    static readonly PointF[] Seta =
    {
        new(0f, 0f), new(0f, 17.5f), new(4.3f, 13.6f), new(7.3f, 20f),
        new(10.4f, 18.5f), new(7.5f, 12.3f), new(12.2f, 11.7f)
    };

    public static void Draw(Graphics g, int size, bool apagado)
    {
        g.SmoothingMode = SmoothingMode.AntiAlias;
        g.PixelOffsetMode = PixelOffsetMode.HighQuality;

        float s = size;
        var fundo = new RectangleF(0, 0, s, s);

        using (var caminho = Arredondado(fundo, s * 0.23f))
        using (var pincel = new LinearGradientBrush(
                   new RectangleF(0, 0, s, s),
                   apagado ? InicioOff : Inicio,
                   apagado ? FimOff : Fim,
                   LinearGradientMode.ForwardDiagonal))
        {
            g.FillPath(pincel, caminho);

            // Brilho de topo: da profundidade sem precisar de sombra (que vira borrao em 16px).
            if (size >= 32)
                using (var luz = new Pen(Color.FromArgb(46, Color.White), Math.Max(1f, s * 0.02f)))
                    g.DrawPath(luz, caminho);
        }

        // Cursor: alto = opaco, base = translucida. Le como "sumindo" sem depender de detalhe fino.
        var altura = s * 0.56f;
        var escala = altura / 20f;
        var largura = 12.2f * escala;
        var origem = new PointF((s - largura) / 2f - s * 0.02f, (s - altura) / 2f);

        var pontos = new PointF[Seta.Length];
        for (var i = 0; i < Seta.Length; i++)
            pontos[i] = new PointF(origem.X + Seta[i].X * escala, origem.Y + Seta[i].Y * escala);

        using var seta = new GraphicsPath();
        seta.AddPolygon(pontos);

        using var desbotado = new LinearGradientBrush(
            new RectangleF(origem.X, origem.Y - 1, largura, altura + 2),
            Color.FromArgb(255, Color.White),
            Color.FromArgb(size >= 32 ? 70 : 130, Color.White), // em 16px o degrade forte some
            LinearGradientMode.Vertical);

        g.FillPath(desbotado, seta);
    }

    static GraphicsPath Arredondado(RectangleF r, float raio)
    {
        var d = raio * 2;
        var p = new GraphicsPath();
        p.AddArc(r.X, r.Y, d, d, 180, 90);
        p.AddArc(r.Right - d, r.Y, d, d, 270, 90);
        p.AddArc(r.Right - d, r.Bottom - d, d, d, 0, 90);
        p.AddArc(r.X, r.Bottom - d, d, d, 90, 90);
        p.CloseFigure();
        return p;
    }

    public static Bitmap Bitmap(int size, bool apagado = false)
    {
        var bmp = new Bitmap(size, size, PixelFormat.Format32bppArgb);
        using var g = Graphics.FromImage(bmp);
        Draw(g, size, apagado);
        return bmp;
    }

    static Icon? _normal, _apagado;

    /// <summary>Icone da bandeja, no tamanho que o Windows pede. Cacheado: o toggle de pausa troca com frequencia.</summary>
    public static Icon Tray(bool apagado)
    {
        var cache = apagado ? _apagado : _normal;
        if (cache != null) return cache;
        var icone = Criar(SystemInformation.SmallIconSize.Width, apagado);
        if (apagado) _apagado = icone; else _normal = icone;
        return icone;
    }

    public static Icon Criar(int size, bool apagado = false)
    {
        using var bmp = Bitmap(size, apagado);
        var h = bmp.GetHicon();
        try
        {
            using var temp = Icon.FromHandle(h);
            return (Icon)temp.Clone();
        }
        finally { DestroyIcon(h); }
    }

    [System.Runtime.InteropServices.DllImport("user32.dll")]
    static extern bool DestroyIcon(IntPtr handle);

    /// <summary>
    /// Escreve um .ico multi-resolucao. DIB classico ate 48px e PNG so no 256 — que e a convencao
    /// que todo mundo consome: PNG em tamanho pequeno o Explorer aceita, mas System.Drawing e
    /// alguns empacotadores (Inno Setup) engasgam.
    /// </summary>
    public static void WriteIco(string caminho, params int[] tamanhos)
    {
        if (tamanhos.Length == 0) tamanhos = new[] { 16, 24, 32, 48, 256 };

        var imagens = new List<byte[]>();
        foreach (var t in tamanhos)
        {
            using var bmp = Bitmap(t);
            if (t >= 256)
            {
                using var ms = new MemoryStream();
                bmp.Save(ms, ImageFormat.Png);
                imagens.Add(ms.ToArray());
            }
            else imagens.Add(Dib(bmp));
        }

        Directory.CreateDirectory(Path.GetDirectoryName(Path.GetFullPath(caminho))!);
        using var fs = File.Create(caminho);
        using var w = new BinaryWriter(fs);

        w.Write((ushort)0);                  // reservado
        w.Write((ushort)1);                  // tipo: icone
        w.Write((ushort)tamanhos.Length);

        var offset = 6 + 16 * tamanhos.Length;
        for (var i = 0; i < tamanhos.Length; i++)
        {
            w.Write((byte)(tamanhos[i] >= 256 ? 0 : tamanhos[i])); // 0 == 256
            w.Write((byte)(tamanhos[i] >= 256 ? 0 : tamanhos[i]));
            w.Write((byte)0);                // paleta
            w.Write((byte)0);                // reservado
            w.Write((ushort)1);              // planos
            w.Write((ushort)32);             // bits por pixel
            w.Write(imagens[i].Length);
            w.Write(offset);
            offset += imagens[i].Length;
        }

        foreach (var img in imagens) w.Write(img);
    }

    /// <summary>BITMAPINFOHEADER + pixels BGRA de baixo para cima + mascara AND zerada (o alfa manda).</summary>
    static byte[] Dib(Bitmap bmp)
    {
        var w = bmp.Width;
        var h = bmp.Height;
        var pixels = new byte[w * h * 4];

        var dados = bmp.LockBits(new Rectangle(0, 0, w, h), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb);
        try
        {
            for (var y = 0; y < h; y++)
            {
                var origem = dados.Scan0 + (h - 1 - y) * dados.Stride; // inverte: DIB e bottom-up
                System.Runtime.InteropServices.Marshal.Copy(origem, pixels, y * w * 4, w * 4);
            }
        }
        finally { bmp.UnlockBits(dados); }

        var mascara = ((w + 31) / 32) * 4 * h; // 1bpp, linhas alinhadas em 4 bytes
        var ms = new MemoryStream();
        var bw = new BinaryWriter(ms);

        bw.Write(40);                       // biSize
        bw.Write(w);                        // biWidth
        bw.Write(h * 2);                    // biHeight: XOR + AND empilhados
        bw.Write((ushort)1);                // biPlanes
        bw.Write((ushort)32);               // biBitCount
        bw.Write(0);                        // biCompression: BI_RGB
        bw.Write(pixels.Length + mascara);  // biSizeImage
        bw.Write(0); bw.Write(0);           // resolucao
        bw.Write(0); bw.Write(0);           // paleta

        bw.Write(pixels);
        bw.Write(new byte[mascara]);
        return ms.ToArray();
    }
}
