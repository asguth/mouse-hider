//! A janela de configuracoes. Cada controle grava na hora: sem OK/Cancelar, menos estado.

pub mod tema;
mod widgets;

use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use eframe::egui::{
    self, Align, CornerRadius, Frame, Key, Layout, Margin, RichText, Stroke, TextureHandle, TextureOptions, vec2,
};

use crate::nucleo::atualizacao::{self, Novidade, Resultado};
use crate::nucleo::config::{Config, OCIOSO_MAX, OCIOSO_MIN};
use crate::nucleo::idiomas::{self, T, t, tf};
use crate::nucleo::plataforma::Plataforma;
use crate::nucleo::{atalho, logo};
use tema::{Paleta, forte};
use widgets::Tipo;

pub const LARGURA: f32 = 560.0;
pub const ALTURA: f32 = 460.0;
const VERSAO: &str = env!("CARGO_PKG_VERSION");
const CAFE: &str = "https://buymeacoffee.com/mousehider";
const OCULTAR: u8 = u8::MAX;

/// Estado dividido entre a janela, a bandeja e as threads do Windows.
pub struct Estado {
    pub config: Mutex<Config>,
    /// Pedido para a janela: 0 = nada, 1..=3 = mostrar na aba + 1, `OCULTAR` = esconder.
    pub abrir: AtomicU8,
}

impl Estado {
    pub fn new(config: Config) -> Self {
        Self { config: Mutex::new(config), abrir: AtomicU8::new(0) }
    }

    pub fn config(&self) -> std::sync::MutexGuard<'_, Config> {
        self.config.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// Mostra a janela na aba dada, ou esconde com None. Quem chama depois acorda o egui.
    pub fn pedir_janela(&self, aba: Option<Aba>) {
        self.abrir.store(aba.map_or(OCULTAR, |a| a as u8 + 1), Ordering::SeqCst);
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u8)]
pub enum Aba {
    Geral = 0,
    Sistema = 1,
    Sobre = 2,
}

/// Cor de uma mensagem de status.
#[derive(Clone, Copy)]
enum Tom {
    Fraco,
    Normal,
    Perigo,
}

enum Busca {
    Parado(Option<(String, Tom)>),
    Verificando,
    TemNova(Novidade),
    Baixando(u32),
    Instalando,
}

enum Msg {
    Verificou(Resultado),
    Progresso(u32),
    Baixou(Option<PathBuf>),
}

pub struct App {
    estado: Arc<Estado>,
    plataforma: Arc<dyn Plataforma>,
    aba: Aba,
    /// O egui nao sabe se a janela esta oculta no Windows; sem isso o contador seguiria
    /// redesenhando na bandeja.
    visivel: bool,
    primeiro_quadro: bool,
    logo: Option<TextureHandle>,
    segundos: String,
    capturando: bool,
    aviso_atalho: bool,
    autostart: bool,
    busca: Busca,
    tx: Sender<Msg>,
    rx: Receiver<Msg>,
}

impl App {
    pub fn new(ctx: &egui::Context, estado: Arc<Estado>, plataforma: Arc<dyn Plataforma>, mostrar: bool) -> Self {
        tema::aplicar(ctx);
        let segundos = estado.config().idle_seconds.to_string();
        let autostart = plataforma.autostart();
        let (tx, rx) = channel();
        Self {
            estado,
            plataforma,
            aba: Aba::Geral,
            visivel: mostrar,
            primeiro_quadro: true,
            logo: None,
            segundos,
            capturando: false,
            aviso_atalho: false,
            autostart,
            busca: Busca::Parado(None),
            tx,
            rx,
        }
    }

    fn logo(&mut self, ctx: &egui::Context) -> egui::TextureId {
        self.logo
            .get_or_insert_with(|| {
                let lado = 96;
                let img = egui::ColorImage::from_rgba_unmultiplied([lado, lado], &logo::rgba(lado as u32, false));
                ctx.load_texture("logo", img, TextureOptions::LINEAR)
            })
            .id()
    }

    /// Pedidos de mostrar/esconder (bandeja, minimizar, segunda instancia) e mensagens das
    /// threads de atualizacao. Toda mudanca de visibilidade passa por aqui.
    fn processar(&mut self, ctx: &egui::Context) {
        match self.estado.abrir.swap(0, Ordering::SeqCst) {
            0 => {}
            OCULTAR => self.mostrar(ctx, false),
            pedido => {
                self.aba = [Aba::Geral, Aba::Sistema, Aba::Sobre][usize::from(pedido - 1).min(2)];
                self.autostart = self.plataforma.autostart();
                self.plataforma.antes_de_mostrar();
                self.mostrar(ctx, true);
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
        }

        while let Ok(msg) = self.rx.try_recv() {
            self.busca = match msg {
                Msg::Verificou(Resultado::TemNova(nova)) => Busca::TemNova(nova),
                Msg::Verificou(Resultado::Atualizado) => Busca::Parado(Some((t(T::Atualizado).into(), Tom::Fraco))),
                Msg::Verificou(Resultado::SemConexao) => Busca::Parado(Some((t(T::FalhaConexao).into(), Tom::Perigo))),
                Msg::Verificou(Resultado::Falhou) => Busca::Parado(Some((t(T::FalhaVerificar).into(), Tom::Perigo))),
                Msg::Progresso(pct) => Busca::Baixando(pct),
                Msg::Baixou(Some(novo)) => {
                    // Troca o exe, abre a versao nova e sai: ela assume daqui.
                    self.busca = Busca::Instalando;
                    if self.plataforma.instalar(&novo) {
                        self.plataforma.sair();
                    }
                    Busca::Parado(Some((t(T::FalhaBaixar).into(), Tom::Perigo)))
                }
                Msg::Baixou(None) => Busca::Parado(Some((t(T::FalhaBaixar).into(), Tom::Perigo))),
            };
        }
    }

    fn mostrar(&mut self, ctx: &egui::Context, visivel: bool) {
        self.visivel = visivel;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(visivel));
    }

    fn alternar_pausa(&mut self) {
        {
            let mut c = self.estado.config();
            c.paused = !c.paused;
            c.salvar();
        }
        self.plataforma.pausa_alterada();
    }

    fn cabecalho(&mut self, ui: &mut egui::Ui, p: &Paleta) {
        let pausado = self.estado.config().paused;
        let logo = self.logo(ui.ctx());
        ui.horizontal(|ui| {
            ui.set_height(44.0);
            ui.add(egui::Image::new((logo, vec2(30.0, 30.0))));
            ui.add_space(4.0);
            ui.vertical(|ui| {
                ui.add_space(3.0);
                ui.spacing_mut().item_spacing.y = 1.0;
                ui.label(RichText::new("Mouse Hider").family(forte()).size(15.5).color(p.texto));
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 6.0;
                    let (ponto, _) = ui.allocate_exact_size(vec2(7.0, 7.0), egui::Sense::hover());
                    ui.painter().circle_filled(ponto.center(), 3.5, if pausado { p.fraco } else { p.marca });
                    let estado = if pausado { t(T::Pausado) } else { t(T::Ativo) };
                    ui.label(RichText::new(estado).size(12.5).color(p.fraco));
                });
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let mut ativo = !pausado;
                let r = widgets::interruptor(ui, p, &mut ativo, t(T::MenuPausar));
                if r.changed() {
                    self.alternar_pausa();
                }
            });
        });
    }

    // --- Geral ---

    fn geral(&mut self, ui: &mut egui::Ui, p: &Paleta) {
        let (limite, pausado, atalho_atual) = {
            let c = self.estado.config();
            (c.idle_seconds, c.paused, c.hotkey.clone())
        };

        widgets::cartao(ui, p, |ui| {
            widgets::linha(ui, p, t(T::EsconderApos), Some((t(T::SegundosSemUso), p.fraco)), |ui| {
                let sufixo = t(T::SegundosCurto);
                if let Some(novo) = widgets::passo(ui, p, &mut self.segundos, limite, (OCIOSO_MIN, OCIOSO_MAX), sufixo) {
                    self.definir_limite(novo);
                }
            });

            let s = t(T::SegundosCurto);
            let m = t(T::MinutosCurto);
            let opcoes: Vec<(String, u32)> = [(3, s), (5, s), (10, s), (30, s), (1, m), (5, m)]
                .iter()
                .map(|(n, u)| (format!("{n} {u}"), if *u == m { n * 60 } else { *n }))
                .collect();
            if let Some(novo) = widgets::segmentado(ui, p, &opcoes, Some(limite)) {
                self.definir_limite(novo);
            }
            ui.add_space(16.0);
            widgets::divisor(ui, p);
            ui.add_space(14.0);

            // Contagem regressiva. Escondido pelo atalho nao zera a ociosidade: por isso
            // pergunta ao cursor, senao a barra seguia contando com o cursor ja invisivel.
            let total = f64::from(limite) * 1000.0;
            let restante = if self.plataforma.cursor_escondido() {
                0.0
            } else {
                (total - f64::from(self.plataforma.ocioso_ms())).max(0.0)
            };
            let (rotulo, valor) = if pausado {
                (t(T::ContadorPausado).to_owned(), String::new())
            } else if restante <= 0.0 {
                (t(T::ContadorEscondido).to_owned(), String::new())
            } else {
                (t(T::Contador).to_owned(), format!("{} {}", idiomas::decimal(restante / 1000.0), t(T::SegundosCurto)))
            };
            ui.horizontal(|ui| {
                ui.label(RichText::new(primeira_maiuscula(&rotulo)).size(13.0).color(p.fraco));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.label(RichText::new(valor).family(forte()).size(13.0).color(p.texto));
                });
            });
            ui.add_space(8.0);
            let fracao = if pausado || total <= 0.0 { 0.0 } else { (restante / total) as f32 };
            widgets::barra(ui, p, fracao, !pausado);
            ui.add_space(18.0);
        });

        ui.add_space(14.0);

        widgets::cartao(ui, p, |ui| {
            let descricao = if self.aviso_atalho {
                Some((t(T::AtalhoOcupado), p.perigo))
            } else if self.capturando {
                Some((t(T::AtalhoDica), p.marca))
            } else {
                None
            };
            widgets::linha(ui, p, t(T::Atalho), descricao, |ui| self.captura(ui, p, &atalho_atual));
        });

        // So o contador precisa de quadros sozinho; o resto redesenha por evento.
        if self.visivel {
            ui.ctx().request_repaint_after(Duration::from_millis(50));
        }
    }

    fn definir_limite(&mut self, segundos: u32) {
        let mut c = self.estado.config();
        c.idle_seconds = segundos.clamp(OCIOSO_MIN, OCIOSO_MAX);
        c.salvar();
        self.segundos = c.idle_seconds.to_string();
    }

    /// Caixa que mostra o atalho como teclas; clicada, escuta a proxima combinacao.
    fn captura(&mut self, ui: &mut egui::Ui, p: &Paleta, atual: &str) {
        let id = ui.scope_id().with("captura-atalho");
        let segurando = ui.input(|i| i.modifiers);
        let partes: Vec<String> = if self.capturando {
            let mut v = Vec::new();
            if segurando.ctrl {
                v.push("Ctrl".to_owned());
            }
            if segurando.alt {
                v.push("Alt".to_owned());
            }
            if segurando.shift {
                v.push("Shift".to_owned());
            }
            v
        } else {
            atalho::partes(atual)
        };

        // Largura exata das teclas: dentro da linha (direita para esquerda) um layout
        // horizontal solto inverteria a ordem ou esticaria ate a borda.
        let largura = if partes.is_empty() {
            28.0
        } else {
            partes.iter().map(|t| widgets::largura_tecla(ui, t)).sum::<f32>() + 4.0 * (partes.len() - 1) as f32
        } + 12.0;
        let quadro = Frame::new()
            .fill(if self.capturando { p.marca_suave } else { p.cartao })
            .stroke(Stroke::new(1.0, if self.capturando { p.marca } else { p.borda }))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(Margin::symmetric(6, 4));
        let resposta = ui
            .allocate_ui_with_layout(vec2(largura, 32.0), Layout::left_to_right(Align::Center), |ui| {
                quadro.show(ui, |ui| {
                    ui.set_min_size(vec2(largura - 12.0, 24.0));
                    ui.spacing_mut().item_spacing.x = 4.0;
                    if partes.is_empty() {
                        ui.add_space(6.0);
                        ui.label(RichText::new("…").color(p.marca));
                    }
                    for parte in &partes {
                        widgets::tecla(ui, p, parte, self.capturando);
                    }
                })
                .response
            })
            .inner;
        let r = ui.interact(resposta.rect, id, egui::Sense::click());
        r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, t(T::Atalho)));
        let r = r.on_hover_cursor(egui::CursorIcon::PointingHand);

        if r.clicked() && !self.capturando {
            self.capturando = true;
            self.aviso_atalho = false;
            self.plataforma.soltar_atalho();
            r.request_focus();
        }
        if !self.capturando {
            if r.has_focus() {
                ui.painter().rect_stroke(r.rect.expand(2.0), CornerRadius::same(10), Stroke::new(2.0, p.marca), egui::StrokeKind::Outside);
                if ui.input(|i| i.key_pressed(Key::Enter) || i.key_pressed(Key::Space)) {
                    self.capturando = true;
                    self.aviso_atalho = false;
                    self.plataforma.soltar_atalho();
                }
            }
            return;
        }

        // Prende o teclado: Tab, setas e Esc chegam aqui em vez de mover o foco.
        ui.memory_mut(|m| {
            m.request_focus(id);
            m.set_focus_lock_filter(id, egui::EventFilter { tab: true, horizontal_arrows: true, vertical_arrows: true, escape: true });
        });

        let mut fim: Option<Option<String>> = None;
        ui.input(|i| {
            for evento in &i.events {
                if let egui::Event::Key { key, pressed: true, modifiers, .. } = evento {
                    if *key == Key::Escape && !modifiers.any() {
                        fim = Some(None);
                    } else if let Some(texto) = atalho::formatar(*key, *modifiers) {
                        fim = Some(Some(texto));
                    }
                }
            }
        });
        // Clique fora cancela.
        if ui.input(|i| i.pointer.any_pressed()) && !r.hovered() && !r.clicked() {
            fim = Some(None);
        }

        let Some(resultado) = fim else { return };
        self.capturando = false;
        ui.memory_mut(|m| m.surrender_focus(id));
        match resultado {
            Some(novo) if self.plataforma.aplicar_atalho(&novo) => {
                let mut c = self.estado.config();
                c.hotkey = novo;
                c.salvar();
            }
            Some(_) => {
                // Combinacao tomada por outro app: volta para a que estava valendo.
                self.aviso_atalho = true;
                self.plataforma.aplicar_atalho(atual);
            }
            None => {
                self.plataforma.aplicar_atalho(atual);
            }
        }
    }

    // --- Sistema ---

    fn sistema(&mut self, ui: &mut egui::Ui, p: &Paleta) {
        widgets::cartao(ui, p, |ui| {
            widgets::linha(ui, p, t(T::IniciarWindows), Some((t(T::IniciarWindowsObs), p.fraco)), |ui| {
                let mut ligado = self.autostart;
                if widgets::interruptor(ui, p, &mut ligado, t(T::IniciarWindows)).changed() {
                    self.plataforma.definir_autostart(ligado);
                    self.autostart = self.plataforma.autostart();
                }
            });
            widgets::divisor(ui, p);
            widgets::linha(ui, p, t(T::Idioma), Some((t(T::IdiomaObs), p.fraco)), |ui| self.lista_idiomas(ui));
            widgets::divisor(ui, p);

            let (descricao, tom) = match &self.busca {
                Busca::Parado(Some((texto, tom))) => (texto.clone(), *tom),
                Busca::Parado(None) => (format!("{} {VERSAO}", t(T::Versao)), Tom::Fraco),
                Busca::Verificando => (t(T::Verificando).to_owned(), Tom::Fraco),
                Busca::TemNova(nova) => (tf(T::NovaVersao, &nova.versao), Tom::Normal),
                Busca::Baixando(pct) => (tf(T::Baixando, pct), Tom::Fraco),
                Busca::Instalando => (t(T::Instalando).to_owned(), Tom::Fraco),
            };
            let cor = match tom {
                Tom::Fraco => p.fraco,
                Tom::Normal => p.marca,
                Tom::Perigo => p.perigo,
            };
            widgets::linha(ui, p, "Mouse Hider", Some((&descricao, cor)), |ui| self.botao_atualizacao(ui, p));
        });
    }

    fn lista_idiomas(&mut self, ui: &mut egui::Ui) {
        let sistema = self.plataforma.idioma_do_sistema();
        let atual = idiomas::codigo();
        let mut escolhido = atual;
        egui::ComboBox::from_id_salt("idioma")
            .width(170.0)
            .height(400.0)
            .selected_text(idiomas::nome_de(atual, sistema))
            .show_ui(ui, |ui| {
                ui.spacing_mut().button_padding = vec2(10.0, 6.0);
                for (codigo, _) in idiomas::DISPONIVEIS {
                    ui.selectable_value(&mut escolhido, codigo, idiomas::nome_de(codigo, sistema));
                }
            });
        if escolhido != atual {
            idiomas::definir(escolhido, sistema);
            let mut c = self.estado.config();
            c.idioma = escolhido.to_owned();
            c.salvar();
            drop(c);
            self.plataforma.idioma_alterado();
            if let Busca::Parado(Some(_)) = self.busca {
                self.busca = Busca::Parado(None); // mensagem antiga ficaria no idioma anterior
            }
        }
    }

    fn botao_atualizacao(&mut self, ui: &mut egui::Ui, p: &Paleta) {
        let ocupado = matches!(self.busca, Busca::Verificando | Busca::Baixando(_) | Busca::Instalando);
        if ocupado {
            ui.add(egui::Spinner::new().size(18.0).color(p.marca));
            return;
        }
        let (texto, tipo) = match self.busca {
            Busca::TemNova(_) => (t(T::Baixar), Tipo::Primario),
            _ => (t(T::BuscarAtualizacao), Tipo::Secundario),
        };
        if !widgets::botao(ui, p, texto, tipo).clicked() {
            return;
        }

        let ctx = ui.ctx().clone();
        let tx = self.tx.clone();
        let plataforma = self.plataforma.clone();
        match &self.busca {
            Busca::TemNova(nova) if nova.url.is_none() => self.plataforma.abrir_url(atualizacao::PAGINA),
            Busca::TemNova(_) if self.plataforma.destino_atualizacao().is_none() => {
                // Exe numa pasta protegida (Arquivos de Programas): sem admin nao da para trocar.
                self.plataforma.abrir_url(atualizacao::PAGINA)
            }
            Busca::TemNova(nova) => {
                let nova = nova.clone();
                self.busca = Busca::Baixando(0);
                std::thread::spawn(move || {
                    let ctx2 = ctx.clone();
                    let tx2 = tx.clone();
                    let mut progresso = move |pct| {
                        let _ = tx2.send(Msg::Progresso(pct));
                        ctx2.request_repaint();
                    };
                    let instalador = atualizacao::baixar(plataforma.as_ref(), &nova, &mut progresso);
                    let _ = tx.send(Msg::Baixou(instalador));
                    ctx.request_repaint();
                });
            }
            _ => {
                self.busca = Busca::Verificando;
                std::thread::spawn(move || {
                    let resultado = atualizacao::verificar(plataforma.as_ref());
                    let _ = tx.send(Msg::Verificou(resultado));
                    ctx.request_repaint();
                });
            }
        }
    }

    // --- Sobre ---

    fn sobre(&mut self, ui: &mut egui::Ui, p: &Paleta) {
        let logo = self.logo(ui.ctx());
        widgets::cartao(ui, p, |ui| {
            ui.add_space(18.0);
            ui.horizontal(|ui| {
                ui.add(egui::Image::new((logo, vec2(48.0, 48.0))));
                ui.add_space(6.0);
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    ui.spacing_mut().item_spacing.y = 2.0;
                    ui.label(RichText::new("Mouse Hider").family(forte()).size(18.0).color(p.texto));
                    ui.label(RichText::new(format!("{} {VERSAO}", t(T::Versao))).size(12.5).color(p.fraco));
                });
            });
            ui.add_space(14.0);
            ui.label(RichText::new(t(T::Descricao)).color(p.texto));
            ui.add_space(14.0);
            Frame::new()
                .fill(p.fundo)
                .corner_radius(CornerRadius::same(8))
                .inner_margin(Margin::symmetric(12, 10))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(RichText::new(t(T::Limitacao)).size(12.5).color(p.fraco));
                });
            ui.add_space(16.0);
            ui.horizontal(|ui| {
                if widgets::botao(ui, p, "GitHub", Tipo::Secundario).clicked() {
                    self.plataforma.abrir_url(atualizacao::REPOSITORIO);
                }
                if widgets::botao(ui, p, "Buy me a coffee", Tipo::Secundario).clicked() {
                    self.plataforma.abrir_url(CAFE);
                }
            });
            ui.add_space(18.0);
        });
    }
}

fn primeira_maiuscula(texto: &str) -> String {
    let mut c = texto.chars();
    c.next().map_or_else(String::new, |p| p.to_uppercase().collect::<String>() + c.as_str())
}

impl eframe::App for App {
    fn clear_color(&self, visuals: &egui::Visuals) -> [f32; 4] {
        visuals.panel_fill.to_normalized_gamma_f32()
    }

    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // O eframe mostra a janela depois do primeiro quadro; aberto pelo login, esconde de novo.
        if std::mem::take(&mut self.primeiro_quadro) && !self.visivel {
            self.mostrar(ctx, false);
        }
        self.processar(ctx);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let p = tema::paleta(ui.ctx());
        Frame::new().fill(p.fundo).inner_margin(Margin { left: 24, right: 24, top: 16, bottom: 20 }).show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_min_height(ui.available_height());
            self.cabecalho(ui, &p);
            ui.add_space(8.0);

            let nomes = [t(T::Geral), t(T::Sistema), t(T::Sobre)];
            if let Some(i) = widgets::abas(ui, &p, &nomes, self.aba as usize) {
                self.aba = [Aba::Geral, Aba::Sistema, Aba::Sobre][i];
            }
            let largura = ui.available_width();
            let linha = ui.cursor().min;
            ui.painter().hline(linha.x..=linha.x + largura, linha.y - ui.spacing().item_spacing.y, Stroke::new(1.0, p.borda));
            ui.add_space(12.0);

            match self.aba {
                Aba::Geral => self.geral(ui, &p),
                Aba::Sistema => self.sistema(ui, &p),
                Aba::Sobre => self.sobre(ui, &p),
            }
        });
    }
}
