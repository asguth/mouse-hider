using System.Drawing.Drawing2D;

namespace MouseHider;

sealed class SettingsForm : Form
{
    readonly Config _config;
    Panel _conteudo = null!;
    readonly List<Button> _navegacao = new();
    readonly List<BotaoChip> _presets = new();
    int _pagina;

    CampoNumero _segundos = null!;
    Label _avisoAtalho = null!;

    // ponytail: sem botao OK/Cancelar — cada controle grava na hora. Menos estado, menos bug.
    public SettingsForm(Config config)
    {
        _config = config;

        FormBorderStyle = FormBorderStyle.FixedDialog;
        MaximizeBox = false;
        MinimizeBox = true;
        StartPosition = FormStartPosition.CenterScreen;
        ClientSize = new Size(640, 400);
        Font = Theme.Fonte();
        Icon = Logo.Criar(32);

        ConstruirUi();
    }

    /// <summary>Monta a janela do zero. Trocar idioma e so chamar isto de novo.</summary>
    void ConstruirUi()
    {
        SuspendLayout();
        Controls.Clear();
        _navegacao.Clear();
        _presets.Clear();

        Text = "Mouse Hider";
        BackColor = Theme.Painel;
        ForeColor = Theme.Texto;

        _conteudo = new Panel { Dock = DockStyle.Fill, Padding = new Padding(28, 24, 28, 24), BackColor = Theme.Painel };
        Controls.Add(_conteudo);
        Controls.Add(Trilho());
        ResumeLayout();

        Abrir(_pagina);
    }

    protected override void OnHandleCreated(EventArgs e)
    {
        base.OnHandleCreated(e);
        Theme.AplicarNaJanela(this);
    }

    /// <summary>
    /// Minimizar some na bandeja em vez de virar botao na barra de tarefas — o app ja vive la,
    /// nao faz sentido ocupar dois lugares. Intercepta o SC_MINIMIZE e nao repassa: assim a
    /// janela nunca chega a ficar iconica e reabrir e um Show() simples, sem estado pendurado.
    /// </summary>
    protected override void WndProc(ref Message m)
    {
        const int WM_SYSCOMMAND = 0x0112, SC_MINIMIZE = 0xF020;
        if (m.Msg == WM_SYSCOMMAND && (m.WParam.ToInt32() & 0xFFF0) == SC_MINIMIZE)
        {
            Hide();
            return;
        }
        base.WndProc(ref m);
    }

    /// <summary>Rede de seguranca para quem minimiza por fora (Win+M, "mostrar area de trabalho").</summary>
    protected override void OnResize(EventArgs e)
    {
        base.OnResize(e);
        if (WindowState != FormWindowState.Minimized) return;
        Hide();
        WindowState = FormWindowState.Normal;
    }

    // --- navegacao lateral (estilo Configuracoes do Win11) ---

    Panel Trilho()
    {
        var trilho = new Panel { Dock = DockStyle.Left, Width = 176, BackColor = Theme.Fundo, Padding = new Padding(10) };

        trilho.Controls.Add(new PictureBox
        {
            Image = Logo.Bitmap(32),
            SizeMode = PictureBoxSizeMode.AutoSize,
            BackColor = Theme.Fundo,
            Location = new Point(18, 22)
        });
        trilho.Controls.Add(new Label
        {
            Text = "Mouse Hider",
            AutoSize = true,
            Font = Theme.Fonte(11f, FontStyle.Bold),
            ForeColor = Theme.Texto,
            BackColor = Theme.Fundo,
            Location = new Point(58, 29)
        });

        var titulos = new[] { Idiomas.T("geral"), Idiomas.T("sistema"), Idiomas.T("sobre") };
        for (var i = 0; i < titulos.Length; i++)
        {
            var indice = i;
            var item = new Button
            {
                Text = "   " + titulos[i],
                TextAlign = ContentAlignment.MiddleLeft,
                FlatStyle = FlatStyle.Flat,
                BackColor = Theme.Fundo,
                ForeColor = Theme.Texto,
                Font = Theme.Fonte(),
                Size = new Size(156, 38),
                Location = new Point(10, 80 + i * 42),
                Cursor = Cursors.Hand
            };
            item.FlatAppearance.BorderSize = 0;
            item.FlatAppearance.MouseOverBackColor = Theme.Hover;
            item.Click += (_, _) => Abrir(indice);
            item.Paint += (remetente, e) =>
            {
                var botao = (Button)remetente!;
                if (botao.Tag as string != "on") return;
                e.Graphics.SmoothingMode = SmoothingMode.AntiAlias;
                using var barra = new SolidBrush(Theme.Accent);
                e.Graphics.FillRectangle(barra, 0, 11, 3, botao.Height - 22);
            };
            _navegacao.Add(item);
            trilho.Controls.Add(item);
        }

        return trilho;
    }

    void Abrir(int indice)
    {
        _pagina = indice;
        for (var i = 0; i < _navegacao.Count; i++)
        {
            _navegacao[i].Tag = i == indice ? "on" : null;
            _navegacao[i].BackColor = i == indice ? Theme.Hover : Theme.Fundo;
            _navegacao[i].Invalidate();
        }

        _conteudo.Controls.Clear();
        _conteudo.Controls.Add(indice switch
        {
            0 => PaginaGeral(),
            1 => PaginaSistema(),
            _ => PaginaSobre()
        });
    }

    // --- Geral ---

    Control PaginaGeral()
    {
        var pagina = NovaPagina(Idiomas.T("geral"));

        pagina.Controls.Add(Texto(Idiomas.T("esconder_apos"), 0, 52, Theme.Texto, Theme.Fonte(10f)));

        _segundos = new CampoNumero
        {
            Minimo = 1,
            Maximo = 3600,
            Sufixo = Idiomas.T("segundos_curto"),
            Location = new Point(0, 78),
            Font = Theme.Fonte(10.5f)
        };
        _segundos.DefinirSemAvisar(_config.IdleSeconds);
        _segundos.ValorAlterado += (_, _) =>
        {
            _config.IdleSeconds = _segundos.Valor;
            _config.Save();
            MarcarPreset();
        };
        pagina.Controls.Add(_segundos);
        pagina.Controls.Add(Texto(Idiomas.T("segundos_sem_uso"), 144, 86, Theme.TextoFraco));

        // FlowLayoutPanel: se a fonte, o idioma ou o DPI mudarem, os presets quebram de linha
        // sozinhos em vez de sumirem na borda.
        var atalhos = new FlowLayoutPanel
        {
            Location = new Point(0, 126),
            MaximumSize = new Size(420, 0),
            AutoSize = true,
            AutoSizeMode = AutoSizeMode.GrowAndShrink,
            BackColor = Theme.Painel,
            WrapContents = true
        };
        var s = Idiomas.T("segundos_curto");
        var min = Idiomas.T("minutos_curto");
        foreach (var (rotulo, valor) in new (string, int)[]
                 { ($"3 {s}", 3), ($"5 {s}", 5), ($"10 {s}", 10), ($"30 {s}", 30), ($"1 {min}", 60), ($"5 {min}", 300) })
        {
            var chip = new BotaoChip(rotulo) { Font = Theme.Fonte(), Tag = valor, Margin = new Padding(0, 0, 7, 7) };
            chip.Ajustar();
            chip.Click += (_, _) => _segundos.Valor = valor;
            _presets.Add(chip);
            atalhos.Controls.Add(chip);
        }
        pagina.Controls.Add(atalhos);
        MarcarPreset();

        // Posicao calculada a partir da altura real dos chips: se eles quebrarem em duas linhas
        // (fonte maior, idioma mais longo), o resto desce junto em vez de ficar por baixo.
        var y = atalhos.Top + atalhos.PreferredSize.Height + 22;

        var rotuloContador = Texto("", 0, y, Theme.TextoFraco);
        var kitt = new ContadorKitt { Location = new Point(0, y + 22), Width = 420 };
        kitt.Estado = () =>
        {
            var total = _config.IdleSeconds * 1000.0;
            var restante = Math.Max(0, total - Native.IdleMilliseconds());
            rotuloContador.Text = _config.Paused
                ? Idiomas.T("contador_pausado")
                : restante <= 0
                    ? Idiomas.T("contador_escondido")
                    : Idiomas.T("contador") + "  " + (restante / 1000.0).ToString("0.0") + " " + Idiomas.T("segundos_curto");
            return (restante, total, _config.Paused);
        };
        pagina.Controls.Add(rotuloContador);
        pagina.Controls.Add(kitt);

        y = kitt.Bottom + 24;
        pagina.Controls.Add(Texto(Idiomas.T("atalho"), 0, y, Theme.Texto, Theme.Fonte(10f)));

        var captura = new CapturaAtalho { Location = new Point(0, y + 26), Font = Theme.Fonte(10f) };
        captura.DefinirSemAvisar(_config.Hotkey);
        _avisoAtalho = Texto(Idiomas.T("atalho_ocupado"), 200, y + 36, Color.FromArgb(0xE0, 0x6C, 0x75));
        _avisoAtalho.Visible = false;

        captura.AtalhoAlterado += (_, _) =>
        {
            if (Program.AplicarAtalho(captura.Atalho))
            {
                _config.Hotkey = captura.Atalho;
                _config.Save();
                _avisoAtalho.Visible = false;
            }
            else
            {
                // Combinacao tomada por outro app: volta para a que estava valendo.
                _avisoAtalho.Visible = true;
                captura.DefinirSemAvisar(_config.Hotkey);
                Program.AplicarAtalho(_config.Hotkey);
            }
        };

        pagina.Controls.Add(captura);
        pagina.Controls.Add(_avisoAtalho);
        return pagina;
    }

    void MarcarPreset()
    {
        foreach (var chip in _presets)
        {
            var selecionado = (int)chip.Tag! == _segundos.Valor;
            if (chip.Selecionado == selecionado) continue;
            chip.Selecionado = selecionado;
            chip.Invalidate();
        }
    }

    // --- Sistema ---

    Control PaginaSistema()
    {
        var pagina = NovaPagina(Idiomas.T("sistema"));

        var chave = new ChaveLigaDesliga { Location = new Point(0, 56) };
        chave.DefinirSemAvisar(Config.AutoStart);
        chave.CheckedChanged += (_, _) => Config.AutoStart = chave.Checked;

        pagina.Controls.Add(chave);
        pagina.Controls.Add(Texto(Idiomas.T("iniciar_windows"), 58, 52, Theme.Texto, Theme.Fonte(10f)));
        pagina.Controls.Add(Texto(Idiomas.T("iniciar_windows_obs"), 58, 74, Theme.TextoFraco));

        pagina.Controls.Add(Texto(Idiomas.T("idioma"), 0, 124, Theme.Texto, Theme.Fonte(10f)));

        var idiomas = new FlowLayoutPanel
        {
            Location = new Point(0, 150),
            Size = new Size(420, 44),
            BackColor = Theme.Painel,
            WrapContents = true
        };
        foreach (var (codigo, nome) in Idiomas.Disponiveis)
        {
            var rotulo = codigo == "auto" ? nome + " (" + Idiomas.DoWindows().ToUpperInvariant() + ")" : nome;
            var chip = new BotaoChip(rotulo)
            {
                Font = Theme.Fonte(),
                Selecionado = Idiomas.Codigo == codigo,
                Margin = new Padding(0, 0, 7, 7)
            };
            chip.Ajustar();
            chip.Click += (_, _) => TrocarIdioma(codigo);
            idiomas.Controls.Add(chip);
        }
        pagina.Controls.Add(idiomas);
        pagina.Controls.Add(Texto(Idiomas.T("idioma_obs"), 0, 200, Theme.TextoFraco));

        var procurar = new BotaoChip(Idiomas.T("buscar_atualizacao")) { Font = Theme.Fonte(), Location = new Point(0, 244) };
        procurar.Ajustar();
        var estadoBusca = Texto("", procurar.Right + 12, 252, Theme.TextoFraco);

        var temNova = false;
        procurar.Click += async (_, _) =>
        {
            // Depois de achar versao nova o mesmo botao vira "Baixar".
            if (temNova) { AbrirNoNavegador(Atualizacao.Pagina); return; }

            procurar.Enabled = false;
            estadoBusca.Text = Idiomas.T("verificando");
            estadoBusca.ForeColor = Theme.TextoFraco;

            var (resultado, versao) = await Atualizacao.Verificar(Versao());
            if (procurar.IsDisposed) return; // usuario pode ter fechado a janela durante a consulta

            switch (resultado)
            {
                case Atualizacao.Resultado.TemNova:
                    temNova = true;
                    estadoBusca.Text = string.Format(Idiomas.T("nova_versao"), versao);
                    estadoBusca.ForeColor = Theme.Texto;
                    procurar.Text = Idiomas.T("baixar");
                    procurar.Selecionado = true;
                    procurar.Ajustar();
                    estadoBusca.Left = procurar.Right + 12;
                    break;
                case Atualizacao.Resultado.Atualizado:
                    estadoBusca.Text = Idiomas.T("atualizado");
                    break;
                default:
                    estadoBusca.Text = Idiomas.T("falha_verificar");
                    break;
            }

            procurar.Enabled = true;
        };

        pagina.Controls.Add(procurar);
        pagina.Controls.Add(estadoBusca);

        return pagina;
    }

    void TrocarIdioma(string codigo)
    {
        if (Idiomas.Codigo == codigo) return;
        Idiomas.Definir(codigo);
        _config.Idioma = codigo;
        _config.Save();
        Program.AtualizarTextos();
        ConstruirUi();
    }

    // --- Sobre ---

    Control PaginaSobre()
    {
        var pagina = NovaPagina(Idiomas.T("sobre"));

        pagina.Controls.Add(new PictureBox
        {
            Image = Logo.Bitmap(56),
            SizeMode = PictureBoxSizeMode.AutoSize,
            Location = new Point(0, 52),
            BackColor = Theme.Painel
        });
        pagina.Controls.Add(Texto("Mouse Hider", 72, 56, Theme.Texto, Theme.Fonte(12f, FontStyle.Bold)));
        pagina.Controls.Add(Texto(Idiomas.T("versao") + " " + Versao(), 72, 82, Theme.TextoFraco));

        pagina.Controls.Add(new Label
        {
            Text = Idiomas.T("descricao") + "\r\n\r\n" + Idiomas.T("limitacao"),
            Location = new Point(0, 124),
            Size = new Size(400, 132),
            ForeColor = Theme.TextoFraco,
            BackColor = Theme.Painel,
            Font = Theme.Fonte()
        });

        // Nome da marca, nao traduz. Sem emoji: o GDI do WinForms desenha ☕ em preto e branco,
        // e o resultado parece sujeira na tela.
        var cafe = new BotaoChip("Buy me a coffee")
        {
            Font = Theme.Fonte(),
            Location = new Point(0, 264),
            Selecionado = true // preenchido em accent: e um botao de acao, nao mais um chip qualquer
        };
        cafe.Ajustar();
        cafe.Click += (_, _) => AbrirNoNavegador("https://buymeacoffee.com/mousehider");
        pagina.Controls.Add(cafe);

        return pagina;
    }

    static string Versao() => Application.ProductVersion.Split('+')[0];

    /// <summary>UseShellExecute e obrigatorio: sem ele o .NET tenta executar a URL como programa.</summary>
    static void AbrirNoNavegador(string url)
    {
        try
        {
            using var _ = System.Diagnostics.Process.Start(
                new System.Diagnostics.ProcessStartInfo(url) { UseShellExecute = true });
        }
        catch (Exception e) { Config.Log("falha ao abrir " + url + ": " + e.Message); }
    }

    // --- pecinhas reaproveitadas pelas tres paginas ---

    Panel NovaPagina(string titulo)
    {
        var pagina = new Panel { Dock = DockStyle.Fill, BackColor = Theme.Painel };
        pagina.Controls.Add(Texto(titulo, 0, 0, Theme.Texto, Theme.Fonte(16f, FontStyle.Bold)));
        return pagina;
    }

    static Label Texto(string texto, int x, int y, Color cor, Font? fonte = null) => new()
    {
        Text = texto,
        AutoSize = true,
        Location = new Point(x, y),
        ForeColor = cor,
        BackColor = Theme.Painel,
        Font = fonte ?? Theme.Fonte()
    };
}
