//! Controles proprios. Os do egui funcionam, mas tem cara de ferramenta de debug; aqui cada
//! peca segue a mesma regua: 32 px de altura, raio 8, borda de 1 px, foco visivel.

use eframe::egui::{
    self, Align, Align2, Color32, Role, CornerRadius, FontId, Frame, Layout, Margin, Rect, Response, RichText, Sense, Stroke,
    StrokeKind, Ui, WidgetInfo, WidgetText, vec2,
};

use super::tema::{Paleta, forte};

pub const ALTURA: f32 = 32.0;
const RAIO: u8 = 8;

/// Anel de foco para navegacao por teclado.
fn foco(ui: &Ui, r: &Response, p: &Paleta, raio: CornerRadius) {
    if r.has_focus() {
        ui.painter().rect_stroke(r.rect.expand(2.0), raio, Stroke::new(2.0, p.marca), StrokeKind::Outside);
    }
}

/// Grupo de linhas de configuracao.
pub fn cartao<R>(ui: &mut Ui, p: &Paleta, conteudo: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::new()
        .fill(p.cartao)
        .stroke(Stroke::new(1.0, p.borda))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(16, 4))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 0.0;
            conteudo(ui)
        })
        .inner
}

pub fn divisor(ui: &mut Ui, p: &Paleta) {
    let (r, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().hline(r.x_range(), r.center().y, Stroke::new(1.0, p.divisor));
}

/// Linha "titulo + descricao a esquerda, controle a direita", centralizada na vertical.
pub fn linha(
    ui: &mut Ui,
    p: &Paleta,
    titulo: &str,
    descricao: Option<(&str, Color32)>,
    controle: impl FnOnce(&mut Ui),
) {
    let largura = ui.available_width();
    ui.allocate_ui_with_layout(vec2(largura, 60.0), Layout::right_to_left(Align::Center), |ui| {
        ui.set_min_height(60.0);
        ui.spacing_mut().item_spacing.x = 16.0;
        controle(ui);

        let resto = ui.available_width();
        let texto = |t: RichText| {
            WidgetText::from(t).into_galley(ui, Some(egui::TextWrapMode::Wrap), resto, egui::TextStyle::Body)
        };
        let g_titulo = texto(RichText::new(titulo).family(forte()).color(p.texto));
        let g_desc = descricao.map(|(d, cor)| texto(RichText::new(d).size(12.5).color(cor)));
        let altura_texto = g_titulo.size().y + g_desc.as_ref().map_or(0.0, |g| g.size().y + 2.0);
        let altura = (altura_texto + 24.0).max(60.0);

        ui.allocate_ui_with_layout(vec2(resto, altura), Layout::top_down(Align::Min), |ui| {
            ui.set_min_size(vec2(resto, altura));
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.add_space((altura - altura_texto) / 2.0);
            ui.label(g_titulo);
            if let Some(g) = g_desc {
                ui.label(g);
            }
        });
    });
}

/// Interruptor liga/desliga.
pub fn interruptor(ui: &mut Ui, p: &Paleta, ligado: &mut bool, rotulo: &str) -> Response {
    let tamanho = vec2(40.0, 22.0);
    let (rect, mut r) = ui.allocate_exact_size(tamanho, Sense::click());
    if r.clicked() {
        *ligado = !*ligado;
        r.mark_changed();
    }
    let valor = *ligado;
    r.widget_info(|| WidgetInfo::selected(Role::Switch, ui.is_enabled(), valor, rotulo));

    let t = ui.ctx().animate_bool_responsive(r.id, *ligado);
    let fundo = if *ligado {
        if r.hovered() { p.marca_hover } else { p.marca }
    } else if r.hovered() {
        p.borda.gamma_multiply(1.15)
    } else {
        p.trilho
    };
    let raio = CornerRadius::same(11);
    let pintor = ui.painter();
    pintor.rect_filled(rect, raio, fundo);
    if !*ligado {
        pintor.rect_stroke(rect, raio, Stroke::new(1.0, p.borda), StrokeKind::Inside);
    }
    let x = egui::lerp(rect.left() + 11.0..=rect.right() - 11.0, t);
    let bola = if *ligado { Color32::WHITE } else { p.cartao };
    pintor.circle_filled(egui::pos2(x, rect.center().y + 0.5), 8.0, Color32::from_black_alpha(30));
    pintor.circle_filled(egui::pos2(x, rect.center().y), 8.0, bola);
    foco(ui, &r, p, raio);
    r.on_hover_cursor(egui::CursorIcon::PointingHand)
}

#[derive(Clone, Copy, PartialEq)]
pub enum Tipo {
    Primario,
    Secundario,
}

pub fn botao(ui: &mut Ui, p: &Paleta, texto: &str, tipo: Tipo) -> Response {
    let galeria = WidgetText::from(RichText::new(texto).family(forte()).size(13.5)).into_galley(
        ui,
        Some(egui::TextWrapMode::Extend),
        f32::INFINITY,
        egui::TextStyle::Button,
    );
    let tamanho = vec2(galeria.size().x + 28.0, ALTURA);
    let (rect, r) = ui.allocate_exact_size(tamanho, Sense::click());
    r.widget_info(|| WidgetInfo::labeled(Role::Button, ui.is_enabled(), texto));

    let habilitado = ui.is_enabled();
    let raio = CornerRadius::same(RAIO);
    let (fundo, borda, cor) = match tipo {
        Tipo::Primario => (if r.hovered() { p.marca_hover } else { p.marca }, None, p.sobre_marca),
        Tipo::Secundario => (if r.hovered() { p.hover } else { p.cartao }, Some(p.borda), p.texto),
    };
    let pintor = ui.painter();
    pintor.rect_filled(rect, raio, fundo);
    if let Some(b) = borda {
        pintor.rect_stroke(rect, raio, Stroke::new(1.0, b), StrokeKind::Inside);
    }
    let cor = if habilitado { cor } else { cor.gamma_multiply(0.45) };
    pintor.galley(rect.center() - galeria.size() / 2.0, galeria, cor);
    foco(ui, &r, p, raio);
    if habilitado { r.on_hover_cursor(egui::CursorIcon::PointingHand) } else { r }
}

/// Controle segmentado: uma escolha entre poucas opcoes. Devolve a clicada.
pub fn segmentado(ui: &mut Ui, p: &Paleta, opcoes: &[(String, u32)], atual: Option<u32>) -> Option<u32> {
    let largura = ui.available_width();
    let (trilho, _) = ui.allocate_exact_size(vec2(largura, ALTURA + 2.0), Sense::hover());
    ui.painter().rect_filled(trilho, CornerRadius::same(RAIO), p.trilho);

    let interno = trilho.shrink(3.0);
    let cada = interno.width() / opcoes.len() as f32;
    let mut clicada = None;
    for (i, (rotulo, valor)) in opcoes.iter().enumerate() {
        let rect = Rect::from_min_size(interno.min + vec2(cada * i as f32, 0.0), vec2(cada, interno.height()));
        let id = ui.scope_id().with(("segmento", i));
        let r = ui.interact(rect, id, Sense::click());
        let selecionado = atual == Some(*valor);
        r.widget_info(|| WidgetInfo::selected(Role::RadioButton, true, selecionado, rotulo));
        if r.clicked() {
            clicada = Some(*valor);
        }
        let raio = CornerRadius::same(RAIO - 2);
        if selecionado {
            ui.painter().rect_filled(rect.translate(vec2(0.0, 1.0)), raio, Color32::from_black_alpha(18));
            ui.painter().rect_filled(rect, raio, p.cartao);
        } else if r.hovered() {
            ui.painter().rect_filled(rect, raio, p.hover.gamma_multiply(0.6));
        }
        let cor = if selecionado || r.hovered() { p.texto } else { p.fraco };
        let fonte = if selecionado { FontId::new(13.0, forte()) } else { FontId::proportional(13.0) };
        ui.painter().text(rect.center(), Align2::CENTER_CENTER, rotulo, fonte, cor);
        foco(ui, &r, p, raio);
        r.on_hover_cursor(egui::CursorIcon::PointingHand);
    }
    clicada
}

/// Campo numerico com - e +. `texto` guarda o que esta sendo digitado entre quadros.
pub fn passo(ui: &mut Ui, p: &Paleta, texto: &mut String, valor: u32, faixa: (u32, u32), sufixo: &str) -> Option<u32> {
    let mut novo = None;
    let raio = CornerRadius::same(RAIO);
    let (caixa, _) = ui.allocate_exact_size(vec2(132.0, ALTURA), Sense::hover());
    ui.painter().rect_filled(caixa, raio, p.cartao);
    ui.painter().rect_stroke(caixa, raio, Stroke::new(1.0, p.borda), StrokeKind::Inside);

    let lado = vec2(ALTURA, ALTURA);
    let menos = Rect::from_min_size(caixa.min, lado);
    let mais = Rect::from_min_size(caixa.right_top() - vec2(ALTURA, 0.0), lado);
    for (rect, simbolo, delta, nome) in [(menos, "−", -1i64, "-"), (mais, "+", 1, "+")] {
        let r = ui.interact(rect, ui.scope_id().with(("passo", nome)), Sense::click());
        r.widget_info(|| WidgetInfo::labeled(Role::Button, true, nome));
        let limite = if delta < 0 { valor <= faixa.0 } else { valor >= faixa.1 };
        if r.hovered() && !limite {
            ui.painter().rect_filled(rect.shrink(3.0), CornerRadius::same(RAIO - 3), p.hover);
        }
        let cor = if limite { p.fraco.gamma_multiply(0.5) } else { p.texto };
        ui.painter().text(rect.center(), Align2::CENTER_CENTER, simbolo, FontId::proportional(17.0), cor);
        if r.clicked() && !limite {
            novo = Some((i64::from(valor) + delta).clamp(i64::from(faixa.0), i64::from(faixa.1)) as u32);
        }
        foco(ui, &r, p, CornerRadius::same(RAIO - 3));
    }

    // Numero + sufixo, centralizados no meio da caixa.
    let meio = Rect::from_min_max(menos.right_top(), mais.left_bottom());
    let fonte = FontId::proportional(14.0);
    let medir = |ui: &Ui, t: &str| ui.fonts_mut(|f| f.layout_no_wrap(t.to_owned(), fonte.clone(), p.texto).size());
    let num = medir(ui, if texto.is_empty() { "0" } else { texto.as_str() });
    let suf = medir(ui, sufixo);
    let total = num.x + 3.0 + suf.x;
    let x0 = meio.center().x - total / 2.0;
    let campo = Rect::from_min_size(egui::pos2(x0, meio.center().y - num.y / 2.0), vec2(num.x + 1.0, num.y));
    let resposta = ui.put(
        campo,
        egui::TextEdit::singleline(texto)
            .frame(Frame::NONE)
            .margin(Margin::ZERO)
            .desired_width(num.x + 1.0)
            .min_size(vec2(0.0, 0.0))
            .char_limit(4)
            .font(fonte.clone()),
    );
    ui.painter().text(
        egui::pos2(x0 + num.x + 3.0, meio.center().y),
        Align2::LEFT_CENTER,
        sufixo,
        fonte.clone(),
        p.fraco,
    );

    if resposta.changed() {
        texto.retain(|c| c.is_ascii_digit());
    }
    let confirmar = resposta.lost_focus();
    if confirmar {
        match texto.parse::<u32>() {
            Ok(n) => novo = Some(n.clamp(faixa.0, faixa.1)),
            Err(_) => *texto = valor.to_string(),
        }
    }
    if let Some(n) = novo {
        *texto = n.to_string();
    } else if !resposta.has_focus() && !confirmar {
        *texto = valor.to_string();
    }
    novo.filter(|n| *n != valor)
}

/// Barra fina de progresso, 0..=1.
pub fn barra(ui: &mut Ui, p: &Paleta, fracao: f32, ativa: bool) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 6.0), Sense::hover());
    let raio = CornerRadius::same(3);
    ui.painter().rect_filled(rect, raio, p.trilho);
    let largura = rect.width() * fracao.clamp(0.0, 1.0);
    if largura > 0.5 {
        let cheio = Rect::from_min_size(rect.min, vec2(largura.max(6.0), rect.height()));
        ui.painter().rect_filled(cheio, raio, if ativa { p.marca } else { p.fraco });
    }
}

/// Largura que `tecla` vai ocupar.
pub fn largura_tecla(ui: &Ui, rotulo: &str) -> f32 {
    let fonte = FontId::new(12.5, forte());
    let largura = ui.fonts_mut(|f| f.layout_no_wrap(rotulo.to_owned(), fonte, Color32::WHITE).size().x);
    (largura + 14.0).max(24.0)
}

/// Uma tecla desenhada como keycap.
pub fn tecla(ui: &mut Ui, p: &Paleta, rotulo: &str, ativa: bool) {
    let fonte = FontId::new(12.5, forte());
    let (rect, _) = ui.allocate_exact_size(vec2(largura_tecla(ui, rotulo), 24.0), Sense::hover());
    let raio = CornerRadius::same(5);
    let pintor = ui.painter();
    pintor.rect_filled(rect.translate(vec2(0.0, 1.5)), raio, p.borda);
    pintor.rect_filled(rect, raio, if ativa { p.marca_suave } else { p.fundo });
    pintor.rect_stroke(rect, raio, Stroke::new(1.0, if ativa { p.marca } else { p.borda }), StrokeKind::Inside);
    pintor.text(rect.center(), Align2::CENTER_CENTER, rotulo, fonte, if ativa { p.marca } else { p.texto });
}

/// Abas sublinhadas. Devolve o indice clicado.
pub fn abas(ui: &mut Ui, p: &Paleta, nomes: &[&str], atual: usize) -> Option<usize> {
    let mut clicada = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 22.0;
        for (i, nome) in nomes.iter().enumerate() {
            let selecionada = i == atual;
            let fonte = FontId::new(14.0, if selecionada { forte() } else { egui::FontFamily::Proportional });
            let largura = ui.fonts_mut(|f| f.layout_no_wrap((*nome).to_owned(), fonte.clone(), p.texto).size().x);
            let (rect, r) = ui.allocate_exact_size(vec2(largura, 38.0), Sense::click());
            r.widget_info(|| WidgetInfo::selected(Role::Tab, true, selecionada, *nome));
            if r.clicked() {
                clicada = Some(i);
            }
            let cor = if selecionada || r.hovered() { p.texto } else { p.fraco };
            ui.painter().text(egui::pos2(rect.left(), rect.center().y - 1.0), Align2::LEFT_CENTER, *nome, fonte, cor);
            if selecionada {
                let base = Rect::from_min_max(egui::pos2(rect.left(), rect.bottom() - 2.0), rect.right_bottom());
                ui.painter().rect_filled(base, CornerRadius::same(1), p.marca);
            }
            foco(ui, &r, p, CornerRadius::same(4));
            r.on_hover_cursor(egui::CursorIcon::PointingHand);
        }
    });
    clicada
}
