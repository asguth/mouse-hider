using System.Drawing.Drawing2D;

namespace MouseHider;

static class Formas
{
    public static GraphicsPath Arredondado(RectangleF r, float raio)
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

    public static GraphicsPath Pilula(RectangleF r) => Arredondado(r, r.Height / 2f);
}

/// <summary>Chave liga/desliga estilo Win11, com animacao curta. Espaco/Enter alternam.</summary>
sealed class ChaveLigaDesliga : Control
{
    bool _ligado;
    float _pos;
    readonly System.Windows.Forms.Timer _anim = new() { Interval = 15 };

    public event EventHandler? CheckedChanged;

    public ChaveLigaDesliga()
    {
        SetStyle(ControlStyles.AllPaintingInWmPaint | ControlStyles.OptimizedDoubleBuffer |
                 ControlStyles.UserPaint | ControlStyles.ResizeRedraw | ControlStyles.Selectable, true);
        TabStop = true;
        Size = new Size(44, 24);
        Cursor = Cursors.Hand;
        _anim.Tick += (_, _) =>
        {
            var alvo = _ligado ? 1f : 0f;
            _pos += (alvo - _pos) * 0.35f;
            if (Math.Abs(alvo - _pos) < 0.01f) { _pos = alvo; _anim.Stop(); }
            Invalidate();
        };
    }

    public bool Checked
    {
        get => _ligado;
        set
        {
            if (_ligado == value) return;
            _ligado = value;
            _anim.Start();
            CheckedChanged?.Invoke(this, EventArgs.Empty);
        }
    }

    /// <summary>Define o estado inicial sem animar nem disparar evento.</summary>
    public void DefinirSemAvisar(bool valor)
    {
        _ligado = valor;
        _pos = valor ? 1f : 0f;
        Invalidate();
    }

    protected override void OnClick(EventArgs e) { Focus(); Checked = !Checked; base.OnClick(e); }

    protected override void OnKeyDown(KeyEventArgs e)
    {
        if (e.KeyCode is Keys.Space or Keys.Enter) { Checked = !Checked; e.Handled = true; }
        base.OnKeyDown(e);
    }

    protected override void OnGotFocus(EventArgs e) { Invalidate(); base.OnGotFocus(e); }
    protected override void OnLostFocus(EventArgs e) { Invalidate(); base.OnLostFocus(e); }

    protected override void OnPaint(PaintEventArgs e)
    {
        var g = e.Graphics;
        g.SmoothingMode = SmoothingMode.AntiAlias;
        g.Clear(Parent?.BackColor ?? Theme.Fundo);

        // Recuo de meia espessura: caneta grossa desenhada rente a borda sai do controle
        // e o clipping come as pontas do traco. Vale para todos os controles deste arquivo.
        const float esp = 1.4f;
        var r = new RectangleF(esp / 2f, esp / 2f, Width - esp, Height - esp);
        using var trilho = Formas.Pilula(r);

        using (var fundo = new SolidBrush(Theme.Misturar(Theme.Fundo, Theme.Accent, _pos)))
            g.FillPath(fundo, trilho);
        using (var borda = new Pen(_pos > 0.5f ? Theme.Accent : Theme.TextoFraco, esp))
            g.DrawPath(borda, trilho);

        var d = r.Height - 8;
        var x = r.X + 4 + _pos * (r.Width - d - 8);
        using (var bolinha = new SolidBrush(_pos > 0.5f ? Color.White : Theme.TextoFraco))
            g.FillEllipse(bolinha, x, r.Y + 4, d, d);

        if (Focused)
            using (var foco = new Pen(Theme.Texto, 1f) { DashStyle = DashStyle.Dot })
                g.DrawRectangle(foco, 0.5f, 0.5f, Width - 1f, Height - 1f);
    }

    protected override void Dispose(bool disposing)
    {
        if (disposing) _anim.Dispose();
        base.Dispose(disposing);
    }
}

/// <summary>
/// Campo numerico proprio. O NumericUpDown do WinForms nao aceita tema — as setinhas continuam
/// claras no modo escuro e a borda e a do sistema.
/// </summary>
sealed class CampoNumero : Control
{
    const int LarguraBotao = 30;
    int _valor = 1;
    int _zonaSobre; // -1 = menos, 1 = mais, 0 = nenhuma

    public int Minimo { get; set; } = 1;
    public int Maximo { get; set; } = 3600;
    public string Sufixo { get; set; } = "";

    public event EventHandler? ValorAlterado;

    public CampoNumero()
    {
        SetStyle(ControlStyles.AllPaintingInWmPaint | ControlStyles.OptimizedDoubleBuffer |
                 ControlStyles.UserPaint | ControlStyles.ResizeRedraw | ControlStyles.Selectable, true);
        TabStop = true;
        Size = new Size(132, 36);
    }

    public int Valor
    {
        get => _valor;
        set
        {
            var novo = Math.Clamp(value, Minimo, Maximo);
            if (novo == _valor) return;
            _valor = novo;
            Invalidate();
            ValorAlterado?.Invoke(this, EventArgs.Empty);
        }
    }

    public void DefinirSemAvisar(int valor)
    {
        _valor = Math.Clamp(valor, Minimo, Maximo);
        Invalidate();
    }

    int ZonaDe(int x) => x >= Width - LarguraBotao ? 1 : x >= Width - LarguraBotao * 2 ? -1 : 0;

    protected override void OnMouseDown(MouseEventArgs e)
    {
        Focus();
        var zona = ZonaDe(e.X);
        if (zona != 0) Valor += zona;
        base.OnMouseDown(e);
    }

    protected override void OnMouseMove(MouseEventArgs e)
    {
        var zona = ZonaDe(e.X);
        if (zona != _zonaSobre) { _zonaSobre = zona; Invalidate(); }
        Cursor = zona == 0 ? Cursors.Default : Cursors.Hand;
        base.OnMouseMove(e);
    }

    protected override void OnMouseLeave(EventArgs e) { _zonaSobre = 0; Invalidate(); base.OnMouseLeave(e); }
    protected override void OnMouseWheel(MouseEventArgs e) { Valor += Math.Sign(e.Delta); base.OnMouseWheel(e); }
    protected override void OnGotFocus(EventArgs e) { Invalidate(); base.OnGotFocus(e); }
    protected override void OnLostFocus(EventArgs e) { Invalidate(); base.OnLostFocus(e); }
    protected override bool IsInputKey(Keys k) => k is Keys.Up or Keys.Down or Keys.PageUp or Keys.PageDown || base.IsInputKey(k);

    protected override void OnKeyDown(KeyEventArgs e)
    {
        switch (e.KeyCode)
        {
            case Keys.Up: Valor += 1; e.Handled = true; break;
            case Keys.Down: Valor -= 1; e.Handled = true; break;
            case Keys.PageUp: Valor += 10; e.Handled = true; break;
            case Keys.PageDown: Valor -= 10; e.Handled = true; break;
        }
        base.OnKeyDown(e);
    }

    protected override void OnPaint(PaintEventArgs e)
    {
        var g = e.Graphics;
        g.SmoothingMode = SmoothingMode.AntiAlias;
        g.Clear(Parent?.BackColor ?? Theme.Painel);

        var esp = Focused ? 1.6f : 1f;
        var r = new RectangleF(esp / 2f, esp / 2f, Width - esp, Height - esp);
        using var caixa = Formas.Arredondado(r, 8);
        using (var fundo = new SolidBrush(Theme.Fundo)) g.FillPath(fundo, caixa);
        using (var borda = new Pen(Focused ? Theme.Accent : Theme.Borda, esp)) g.DrawPath(borda, caixa);

        var texto = Sufixo.Length > 0 ? _valor + " " + Sufixo : _valor.ToString();
        TextRenderer.DrawText(g, texto, Font, new Rectangle(12, 0, Width - LarguraBotao * 2 - 12, Height),
            Theme.Texto, TextFormatFlags.VerticalCenter | TextFormatFlags.Left);

        DesenhaBotao(g, caixa, Width - LarguraBotao * 2, -1);
        DesenhaBotao(g, caixa, Width - LarguraBotao, 1);
    }

    void DesenhaBotao(Graphics g, GraphicsPath caixa, int x, int sinal)
    {
        var area = new Rectangle(x, 1, LarguraBotao, Height - 2);
        // Recorta na caixa: o realce e quadrado e vazaria pelo canto arredondado da direita.
        if (_zonaSobre == sinal)
        {
            g.SetClip(caixa);
            using (var realce = new SolidBrush(Theme.Hover)) g.FillRectangle(realce, area);
            g.ResetClip();
        }

        var habilitado = sinal > 0 ? _valor < Maximo : _valor > Minimo;
        using var caneta = new Pen(habilitado ? Theme.Texto : Theme.TextoFraco, 1.6f);
        var cx = area.X + area.Width / 2f;
        var cy = area.Y + area.Height / 2f;
        g.DrawLine(caneta, cx - 5, cy, cx + 5, cy);
        if (sinal > 0) g.DrawLine(caneta, cx, cy - 5, cx, cy + 5);
    }
}

/// <summary>Botao de preset em formato de pilula, com estado selecionado.</summary>
sealed class BotaoChip : Control
{
    bool _sobre;

    public bool Selecionado { get; set; }

    public BotaoChip(string texto)
    {
        SetStyle(ControlStyles.AllPaintingInWmPaint | ControlStyles.OptimizedDoubleBuffer |
                 ControlStyles.UserPaint | ControlStyles.ResizeRedraw | ControlStyles.Selectable, true);
        TabStop = true;
        Text = texto;
        Cursor = Cursors.Hand;
        Height = 32;
        Width = 0; // definido em Ajustar()
    }

    /// <summary>Largura pelo texto renderizado: nao quebra quando o idioma ou o DPI mudam.</summary>
    public void Ajustar(int minimo = 56)
    {
        var largura = TextRenderer.MeasureText(Text, Font).Width + 26;
        Width = Math.Max(minimo, largura);
    }

    protected override void OnMouseEnter(EventArgs e) { _sobre = true; Invalidate(); base.OnMouseEnter(e); }
    protected override void OnMouseLeave(EventArgs e) { _sobre = false; Invalidate(); base.OnMouseLeave(e); }
    protected override void OnClick(EventArgs e) { Focus(); base.OnClick(e); }
    protected override void OnGotFocus(EventArgs e) { Invalidate(); base.OnGotFocus(e); }
    protected override void OnLostFocus(EventArgs e) { Invalidate(); base.OnLostFocus(e); }

    protected override void OnKeyDown(KeyEventArgs e)
    {
        if (e.KeyCode is Keys.Space or Keys.Enter) { InvokeOnClick(this, EventArgs.Empty); e.Handled = true; }
        base.OnKeyDown(e);
    }

    protected override void OnPaint(PaintEventArgs e)
    {
        var g = e.Graphics;
        g.SmoothingMode = SmoothingMode.AntiAlias;
        g.Clear(Parent?.BackColor ?? Theme.Painel);

        var esp = Focused ? 1.6f : 1f;
        var r = new RectangleF(esp / 2f, esp / 2f, Width - esp, Height - esp);
        using var pilula = Formas.Pilula(r);

        // Selecionado tambem responde ao hover: o "Buy me a coffee" e um chip selecionado
        // e sem isto o botao de acao mais visivel da tela nao reage ao mouse.
        var fundo = Selecionado
            ? _sobre ? Theme.Misturar(Theme.Accent, Color.White, 0.16f) : Theme.Accent
            : _sobre ? Theme.Hover : Theme.Fundo;
        using (var pincel = new SolidBrush(fundo)) g.FillPath(pincel, pilula);
        if (!Selecionado)
            using (var borda = new Pen(Focused ? Theme.Accent : Theme.Borda, esp))
                g.DrawPath(borda, pilula);

        TextRenderer.DrawText(g, Text, Font, new Rectangle(0, 0, Width, Height),
            Selecionado ? Color.White : Theme.Texto,
            TextFormatFlags.HorizontalCenter | TextFormatFlags.VerticalCenter);
    }
}

/// <summary>Caixa que captura uma combinacao de teclas: clique nela e pressione o atalho.</summary>
sealed class CapturaAtalho : Control
{
    bool _capturando;

    public string Atalho { get; private set; } = "Ctrl+Alt+H";
    public event EventHandler? AtalhoAlterado;

    public CapturaAtalho()
    {
        SetStyle(ControlStyles.AllPaintingInWmPaint | ControlStyles.OptimizedDoubleBuffer |
                 ControlStyles.UserPaint | ControlStyles.ResizeRedraw | ControlStyles.Selectable, true);
        TabStop = true;
        Height = 36;
        Cursor = Cursors.Hand;
        Ajustar();
    }

    /// <summary>
    /// Largura pelo maior dos dois textos que a caixa mostra. Com 190 px fixos a dica
    /// nao cabia nem em portugues â em alemao seria pior.
    /// </summary>
    public void Ajustar()
    {
        var dica = TextRenderer.MeasureText(Idiomas.T("atalho_dica"), Font).Width;
        var atalho = TextRenderer.MeasureText(Atalho, Font).Width;
        Width = Math.Max(190, Math.Max(dica, atalho) + 32);
    }

    protected override void OnFontChanged(EventArgs e) { base.OnFontChanged(e); Ajustar(); }

    public void DefinirSemAvisar(string atalho)
    {
        if (!string.IsNullOrWhiteSpace(atalho)) Atalho = atalho;
        Ajustar();
        Invalidate();
    }

    protected override void OnClick(EventArgs e) { Focus(); _capturando = true; Invalidate(); base.OnClick(e); }
    protected override void OnLostFocus(EventArgs e) { _capturando = false; Invalidate(); base.OnLostFocus(e); }
    protected override void OnGotFocus(EventArgs e) { Invalidate(); base.OnGotFocus(e); }

    // ProcessCmdKey em vez de OnKeyDown: e o unico ponto que ve Alt e Ctrl antes do menu do sistema.
    protected override bool ProcessCmdKey(ref Message msg, Keys keyData)
    {
        if (!_capturando) return base.ProcessCmdKey(ref msg, keyData);

        var tecla = keyData & Keys.KeyCode;
        if (tecla == Keys.Escape) { _capturando = false; Invalidate(); return true; }

        // Modificador sozinho ainda nao e um atalho; espera a tecla final.
        if (tecla is Keys.ControlKey or Keys.Menu or Keys.ShiftKey or Keys.None) return true;

        var texto = MouseHider.Atalho.Formatar(keyData);
        if (texto == null) return true; // sem modificador: recusa, senao rouba teclas soltas do sistema

        Atalho = texto;
        _capturando = false;
        Ajustar();
        Invalidate();
        AtalhoAlterado?.Invoke(this, EventArgs.Empty);
        return true;
    }

    protected override void OnPaint(PaintEventArgs e)
    {
        var g = e.Graphics;
        g.SmoothingMode = SmoothingMode.AntiAlias;
        g.Clear(Parent?.BackColor ?? Theme.Painel);

        var esp = _capturando ? 1.8f : Focused ? 1.6f : 1f;
        var r = new RectangleF(esp / 2f, esp / 2f, Width - esp, Height - esp);
        using var caixa = Formas.Arredondado(r, 8);
        using (var fundo = new SolidBrush(Theme.Fundo)) g.FillPath(fundo, caixa);
        using (var borda = new Pen(_capturando || Focused ? Theme.Accent : Theme.Borda, esp))
            g.DrawPath(borda, caixa);

        TextRenderer.DrawText(g, _capturando ? Idiomas.T("atalho_dica") : Atalho, Font,
            new Rectangle(0, 0, Width, Height),
            _capturando ? Theme.TextoFraco : Theme.Texto,
            TextFormatFlags.HorizontalCenter | TextFormatFlags.VerticalCenter);
    }
}

/// <summary>Conversao entre "Ctrl+Alt+H" e os codigos que o RegisterHotKey espera.</summary>
static class Atalho
{
    public const int ModAlt = 0x0001, ModControl = 0x0002, ModShift = 0x0004, ModNoRepeat = 0x4000;

    /// <summary>Texto do atalho, ou null se nao houver ao menos um modificador.</summary>
    public static string? Formatar(Keys keyData)
    {
        var partes = new List<string>();
        if (keyData.HasFlag(Keys.Control)) partes.Add("Ctrl");
        if (keyData.HasFlag(Keys.Alt)) partes.Add("Alt");
        if (keyData.HasFlag(Keys.Shift)) partes.Add("Shift");
        if (partes.Count == 0) return null;

        partes.Add(NomeDaTecla(keyData & Keys.KeyCode));
        return string.Join("+", partes);
    }

    static string NomeDaTecla(Keys tecla) => tecla switch
    {
        >= Keys.D0 and <= Keys.D9 => ((char)('0' + (tecla - Keys.D0))).ToString(),
        >= Keys.NumPad0 and <= Keys.NumPad9 => "Num" + (tecla - Keys.NumPad0),
        _ => tecla.ToString()
    };

    /// <summary>(modificadores, virtual key) para o RegisterHotKey, ou null se o texto nao servir.</summary>
    public static (int Mods, int Vk)? Interpretar(string? texto)
    {
        if (string.IsNullOrWhiteSpace(texto)) return null;

        var mods = 0;
        Keys tecla = Keys.None;

        foreach (var parte in texto.Split('+', StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries))
        {
            switch (parte.ToLowerInvariant())
            {
                case "ctrl" or "control": mods |= ModControl; break;
                case "alt": mods |= ModAlt; break;
                case "shift": mods |= ModShift; break;
                default:
                    var nome = parte.Length == 1 && char.IsDigit(parte[0]) ? "D" + parte : parte;
                    if (!Enum.TryParse(nome, true, out tecla)) return null;
                    break;
            }
        }

        if (mods == 0 || tecla == Keys.None) return null;
        return (mods | ModNoRepeat, (int)tecla);
    }
}

/// <summary>
/// Contador regressivo ate esconder o cursor, com varredura estilo KITT: 18 lampadas em
/// vaivem, com rastro. A varredura acelera conforme o tempo acaba — sem isso a barra fica
/// com a mesma cara faltando 30 s ou 300 ms.
/// </summary>
sealed class ContadorKitt : Control
{
    const int Lampadas = 18;
    const int Margem = 6;

    readonly System.Windows.Forms.Timer _tique = new() { Interval = 33 }; // ~30 fps
    float _posicao;          // 0..Lampadas-1, onde esta o foco da varredura
    int _sentido = 1;
    float _brilhoPausa;      // respiro lento quando esta pausado ou ja escondido

    /// <summary>Devolve (ms restantes ate esconder, ms totais, pausado). Restante 0 = escondido.</summary>
    public Func<(double Restante, double Total, bool Pausado)>? Estado { get; set; }

    public ContadorKitt()
    {
        SetStyle(ControlStyles.AllPaintingInWmPaint | ControlStyles.OptimizedDoubleBuffer |
                 ControlStyles.UserPaint | ControlStyles.ResizeRedraw, true);
        Height = 30;
        _tique.Tick += (_, _) => Avancar();
    }

    // So anima com a janela na frente: o app passa 99% do tempo escondido na bandeja.
    // OnHandleCreated tambem, e nao so VisibleChanged: ao trocar de aba o controle nasce
    // dentro de um pai ja visivel, nao ha transicao de visibilidade, o evento nunca
    // dispara e a varredura ficava parada ate reabrir a janela.
    protected override void OnVisibleChanged(EventArgs e) { base.OnVisibleChanged(e); SincronizarTimer(); }
    protected override void OnHandleCreated(EventArgs e) { base.OnHandleCreated(e); SincronizarTimer(); }

    void SincronizarTimer() => _tique.Enabled = Visible;

    void Avancar()
    {
        var (restante, total, pausado) = Estado?.Invoke() ?? (0, 1, true);
        var faltando = total > 0 ? Math.Clamp(restante / total, 0, 1) : 0;

        if (pausado || restante <= 0)
        {
            // Parado no meio, respirando devagar.
            _posicao += (Lampadas / 2f - 0.5f - _posicao) * 0.15f;
            _brilhoPausa += 0.045f;
        }
        else
        {
            // 0.18 lampada por quadro parado, ate ~0.75 no fim da contagem.
            var velocidade = 0.18f + (1f - (float)faltando) * 0.57f;
            _posicao += velocidade * _sentido;
            if (_posicao >= Lampadas - 1) { _posicao = Lampadas - 1; _sentido = -1; }
            else if (_posicao <= 0) { _posicao = 0; _sentido = 1; }
            _brilhoPausa = 0;
        }

        Invalidate();
    }

    protected override void OnPaint(PaintEventArgs e)
    {
        var g = e.Graphics;
        g.SmoothingMode = SmoothingMode.AntiAlias;
        g.Clear(Parent?.BackColor ?? Theme.Painel);

        var (restante, total, pausado) = Estado?.Invoke() ?? (0, 1, true);
        var escondido = restante <= 0 && !pausado;

        var trilho = new RectangleF(0.5f, 0.5f, Width - 1, Height - 1);
        using (var fundo = new SolidBrush(Theme.Fundo)) using (var caminho = Formas.Arredondado(trilho, 8))
        {
            g.FillPath(fundo, caminho);
            using var borda = new Pen(Theme.Borda, 1f);
            g.DrawPath(borda, caminho);
        }

        var largura = (Width - Margem * 2) / (float)Lampadas;
        var altura = Height - Margem * 2;
        var atenuacao = pausado ? 0.25f : escondido ? 0.55f : 1f;
        // Respiro do estado parado: some e volta em vez de ficar cravado.
        if (pausado || escondido) atenuacao *= 0.55f + 0.45f * (float)Math.Abs(Math.Sin(_brilhoPausa));

        for (var i = 0; i < Lampadas; i++)
        {
            // Rastro: cai rapido nas duas lampadas vizinhas e some na quarta.
            var distancia = Math.Abs(i - _posicao);
            var intensidade = Math.Max(0f, 1f - distancia / 3.4f);
            intensidade *= intensidade * atenuacao;
            if (intensidade < 0.02f) continue;

            var cor = Theme.Misturar(Theme.Accent, Color.White, Math.Max(0f, intensidade - 0.72f) * 2.2f);
            using var lampada = new SolidBrush(Color.FromArgb((int)(255 * intensidade), cor));
            var r = new RectangleF(Margem + i * largura + 1, Margem, largura - 2, altura);
            using var forma = Formas.Arredondado(r, 2.5f);
            g.FillPath(lampada, forma);
        }
    }

    protected override void Dispose(bool disposing)
    {
        if (disposing) _tique.Dispose();
        base.Dispose(disposing);
    }
}

/// <summary>Roda de carregamento: arco girando sobre um anel fraco.</summary>
sealed class Girador : Control
{
    readonly System.Windows.Forms.Timer _tique = new() { Interval = 33 }; // ~30 fps
    float _angulo;

    public Girador()
    {
        SetStyle(ControlStyles.AllPaintingInWmPaint | ControlStyles.OptimizedDoubleBuffer |
                 ControlStyles.UserPaint | ControlStyles.ResizeRedraw, true);
        Size = new Size(20, 20);
        _tique.Tick += (_, _) => { _angulo = (_angulo + 11f) % 360f; Invalidate(); };
    }

    // Mesmo cuidado do ContadorKitt: OnHandleCreated cobre o controle criado dentro de um
    // pai que ja estava visivel, caso em que VisibleChanged nunca dispara.
    protected override void OnVisibleChanged(EventArgs e) { base.OnVisibleChanged(e); SincronizarTimer(); }
    protected override void OnHandleCreated(EventArgs e) { base.OnHandleCreated(e); SincronizarTimer(); }

    void SincronizarTimer() => _tique.Enabled = Visible;

    protected override void OnPaint(PaintEventArgs e)
    {
        var g = e.Graphics;
        g.SmoothingMode = SmoothingMode.AntiAlias;
        g.Clear(Parent?.BackColor ?? Theme.Painel);

        const float esp = 2.4f;
        var r = new RectangleF(esp / 2f, esp / 2f, Width - esp, Height - esp);
        using (var anel = new Pen(Theme.Borda, esp)) g.DrawEllipse(anel, r);
        using var arco = new Pen(Theme.Accent, esp) { StartCap = LineCap.Round, EndCap = LineCap.Round };
        g.DrawArc(arco, r, _angulo, 100f);
    }

    protected override void Dispose(bool disposing)
    {
        if (disposing) _tique.Dispose();
        base.Dispose(disposing);
    }
}

/// <summary>
/// Dropdown no tema do app. ponytail: a lista aberta e um ContextMenuStrip com o
/// Theme.MenuRenderer que ja existe para o menu da bandeja — o ComboBox do WinForms nao
/// aceita tema (borda e seta continuam claras no modo escuro) e um popup proprio seria
/// mais uma janela sem foco para gerenciar.
/// </summary>
sealed class ListaSuspensa : Control
{
    readonly ContextMenuStrip _menu;
    readonly (string Valor, string Rotulo)[] _itens;
    bool _sobre;

    public string Selecionado { get; private set; }
    public event EventHandler? SelecaoAlterada;

    public ListaSuspensa((string Valor, string Rotulo)[] itens, string selecionado)
    {
        SetStyle(ControlStyles.AllPaintingInWmPaint | ControlStyles.OptimizedDoubleBuffer |
                 ControlStyles.UserPaint | ControlStyles.ResizeRedraw | ControlStyles.Selectable, true);
        TabStop = true;
        _itens = itens;
        Selecionado = selecionado;
        Size = new Size(220, 36);
        Cursor = Cursors.Hand;

        // ShowCheckMargin sem ShowImageMargin: sobra so a coluna do tique, sem a faixa
        // larga de icones que o ToolStrip desenha por padrao.
        _menu = new ContextMenuStrip
        {
            Renderer = new Theme.MenuRenderer(),
            BackColor = Theme.Painel,
            ForeColor = Theme.Texto,
            ShowImageMargin = false,
            ShowCheckMargin = true
        };
        foreach (var (valor, rotulo) in itens)
        {
            var item = new ToolStripMenuItem(rotulo) { Tag = valor };
            item.Click += (_, _) => Escolher(valor);
            _menu.Items.Add(item);
        }
    }

    void Escolher(string valor)
    {
        if (valor == Selecionado) return;
        Selecionado = valor;
        Invalidate();
        SelecaoAlterada?.Invoke(this, EventArgs.Empty);
    }

    string RotuloAtual() => _itens.FirstOrDefault(i => i.Valor == Selecionado).Rotulo ?? "";

    protected override void OnMouseEnter(EventArgs e) { _sobre = true; Invalidate(); base.OnMouseEnter(e); }
    protected override void OnMouseLeave(EventArgs e) { _sobre = false; Invalidate(); base.OnMouseLeave(e); }
    protected override void OnGotFocus(EventArgs e) { Invalidate(); base.OnGotFocus(e); }
    protected override void OnLostFocus(EventArgs e) { Invalidate(); base.OnLostFocus(e); }
    protected override void OnClick(EventArgs e) { Focus(); Abrir(); base.OnClick(e); }
    protected override bool IsInputKey(Keys k) => k is Keys.Down || base.IsInputKey(k);

    protected override void OnKeyDown(KeyEventArgs e)
    {
        if (e.KeyCode is Keys.Space or Keys.Enter or Keys.Down) { Abrir(); e.Handled = true; }
        base.OnKeyDown(e);
    }

    void Abrir()
    {
        _menu.Font = Font;
        foreach (ToolStripMenuItem item in _menu.Items) item.Checked = (string)item.Tag! == Selecionado;
        _menu.Show(this, new Point(0, Height + 2));
    }

    protected override void OnPaint(PaintEventArgs e)
    {
        var g = e.Graphics;
        g.SmoothingMode = SmoothingMode.AntiAlias;
        g.Clear(Parent?.BackColor ?? Theme.Painel);

        var esp = Focused ? 1.6f : 1f;
        var r = new RectangleF(esp / 2f, esp / 2f, Width - esp, Height - esp);
        using var caixa = Formas.Arredondado(r, 8);
        using (var fundo = new SolidBrush(_sobre ? Theme.Hover : Theme.Fundo)) g.FillPath(fundo, caixa);
        using (var borda = new Pen(Focused ? Theme.Accent : Theme.Borda, esp)) g.DrawPath(borda, caixa);

        TextRenderer.DrawText(g, RotuloAtual(), Font, new Rectangle(12, 0, Width - 42, Height),
            Theme.Texto, TextFormatFlags.VerticalCenter | TextFormatFlags.Left | TextFormatFlags.EndEllipsis);

        using var seta = new Pen(Theme.TextoFraco, 1.6f) { StartCap = LineCap.Round, EndCap = LineCap.Round };
        var cx = Width - 18f;
        var cy = Height / 2f - 1f;
        g.DrawLines(seta, new[] { new PointF(cx - 4, cy), new PointF(cx, cy + 4), new PointF(cx + 4, cy) });
    }

    protected override void Dispose(bool disposing)
    {
        if (disposing) _menu.Dispose();
        base.Dispose(disposing);
    }
}
