//! Paleta, estilo do egui e fontes do sistema.
//!
//! Cinzas levemente quentes e uma unica cor de marca. Nada de degrade, brilho ou sombra
//! colorida: a hierarquia vem de tamanho, peso e espaco.

use std::sync::Arc;

use eframe::egui::{
    self, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Margin, Shadow, Stroke, TextStyle, Theme,
    Visuals,
};

#[derive(Clone, Copy)]
pub struct Paleta {
    pub fundo: Color32,
    pub cartao: Color32,
    pub borda: Color32,
    pub divisor: Color32,
    pub texto: Color32,
    pub fraco: Color32,
    pub hover: Color32,
    pub trilho: Color32,
    pub marca: Color32,
    pub marca_hover: Color32,
    pub marca_suave: Color32,
    pub sobre_marca: Color32,
    pub perigo: Color32,
}

const fn hex(c: u32) -> Color32 {
    Color32::from_rgb((c >> 16) as u8, (c >> 8) as u8, c as u8)
}

pub const CLARA: Paleta = Paleta {
    fundo: hex(0xF4F4F2),
    cartao: hex(0xFFFFFF),
    borda: hex(0xE3E3DF),
    divisor: hex(0xEDEDEA),
    texto: hex(0x1C1C1A),
    fraco: hex(0x6B6B66),
    hover: hex(0xEFEFEC),
    trilho: hex(0xE7E7E3),
    marca: hex(0x0E7C70),
    marca_hover: hex(0x0B6A60),
    marca_suave: hex(0xE2F1EE),
    sobre_marca: hex(0xFFFFFF),
    perigo: hex(0xC4323A),
};

pub const ESCURA: Paleta = Paleta {
    fundo: hex(0x141414),
    cartao: hex(0x1C1C1C),
    borda: hex(0x2C2C2B),
    divisor: hex(0x262626),
    texto: hex(0xECECEA),
    fraco: hex(0x95958F),
    hover: hex(0x262626),
    trilho: hex(0x2E2E2D),
    marca: hex(0x3BB5A3),
    marca_hover: hex(0x55C7B6),
    marca_suave: hex(0x16302C),
    sobre_marca: hex(0x06201C),
    perigo: hex(0xF07076),
};

pub fn paleta(ctx: &egui::Context) -> Paleta {
    match ctx.theme() {
        Theme::Dark => ESCURA,
        Theme::Light => CLARA,
    }
}

/// Familia do texto em semibold (titulos, rotulos de linha). egui nao tem peso de fonte:
/// e outra familia, apontando para o arquivo semibold.
pub fn forte() -> FontFamily {
    FontFamily::Name("forte".into())
}

pub fn aplicar(ctx: &egui::Context) {
    ctx.options_mut(|o| o.theme_preference = egui::ThemePreference::System);
    ctx.set_visuals_of(Theme::Light, visuals(&CLARA, false));
    ctx.set_visuals_of(Theme::Dark, visuals(&ESCURA, true));
    ctx.all_styles_mut(|s| {
        s.text_styles = [
            (TextStyle::Small, FontId::proportional(12.0)),
            (TextStyle::Body, FontId::proportional(14.0)),
            (TextStyle::Button, FontId::proportional(14.0)),
            (TextStyle::Monospace, FontId::monospace(13.0)),
            (TextStyle::Heading, FontId::new(20.0, forte())),
        ]
        .into();
        s.spacing.item_spacing = egui::vec2(8.0, 8.0);
        s.spacing.button_padding = egui::vec2(12.0, 6.0);
        s.spacing.interact_size.y = 32.0;
        s.spacing.combo_height = 320.0;
        s.spacing.menu_margin = Margin::same(6);
        s.interaction.selectable_labels = false;
        s.animation_time = 0.12;
    });
    fontes(ctx);
}

fn visuals(p: &Paleta, escuro: bool) -> Visuals {
    let mut v = if escuro { Visuals::dark() } else { Visuals::light() };
    let raio = CornerRadius::same(8);
    v.panel_fill = p.fundo;
    v.window_fill = p.cartao;
    v.window_stroke = Stroke::new(1.0, p.borda);
    v.window_corner_radius = CornerRadius::same(10);
    v.menu_corner_radius = CornerRadius::same(10);
    v.extreme_bg_color = p.cartao;
    v.faint_bg_color = p.hover;
    v.override_text_color = Some(p.texto);
    v.hyperlink_color = p.marca;
    v.error_fg_color = p.perigo;
    v.selection.bg_fill = p.marca_suave;
    v.selection.stroke = Stroke::new(1.0, p.marca);
    v.text_cursor.stroke = Stroke::new(1.5, p.marca);
    v.popup_shadow = Shadow { offset: [0, 6], blur: 18, spread: 0, color: Color32::from_black_alpha(if escuro { 110 } else { 28 }) };
    v.window_shadow = v.popup_shadow;

    let w = &mut v.widgets;
    for estado in [&mut w.noninteractive, &mut w.inactive, &mut w.hovered, &mut w.active, &mut w.open] {
        estado.corner_radius = raio;
        estado.fg_stroke = Stroke::new(1.0, p.texto);
        estado.expansion = 0.0;
    }
    w.noninteractive.bg_fill = p.cartao;
    w.noninteractive.weak_bg_fill = p.cartao;
    w.noninteractive.bg_stroke = Stroke::new(1.0, p.divisor);
    w.inactive.bg_fill = p.cartao;
    w.inactive.weak_bg_fill = p.cartao;
    w.inactive.bg_stroke = Stroke::new(1.0, p.borda);
    w.hovered.bg_fill = p.hover;
    w.hovered.weak_bg_fill = p.hover;
    w.hovered.bg_stroke = Stroke::new(1.0, p.borda);
    w.active.bg_fill = p.hover;
    w.active.weak_bg_fill = p.hover;
    w.active.bg_stroke = Stroke::new(1.0, p.marca);
    w.open.bg_fill = p.cartao;
    w.open.weak_bg_fill = p.cartao;
    w.open.bg_stroke = Stroke::new(1.0, p.marca);
    v
}

/// Fontes do proprio sistema, lidas por mapeamento de memoria: nada embutido no exe e o
/// arquivo de 20 MB da fonte chinesa nao vira 20 MB de RAM.
fn fontes(ctx: &egui::Context) {
    let mut defs = FontDefinitions::default();
    let mut regular = Vec::new();
    let mut forte_ = Vec::new();

    for (nome, candidatos, peso_forte) in candidatas() {
        let Some(dados) = candidatos.iter().find_map(|c| mapear(c)) else { continue };
        defs.font_data.insert(nome.to_owned(), Arc::new(dados));
        if peso_forte { forte_.push(nome.to_owned()) } else { regular.push(nome.to_owned()) }
    }

    // A primeira regular encontrada vira a principal; as demais sao fallback para CJK e simbolos.
    let fallback_padrao = defs.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let mut proporcional = regular.clone();
    proporcional.extend(fallback_padrao.iter().cloned());
    let mut forte_familia = forte_;
    forte_familia.extend(proporcional.iter().cloned());

    defs.families.insert(FontFamily::Proportional, proporcional);
    defs.families.insert(forte(), forte_familia);
    ctx.set_fonts(defs);
}

/// (nome, arquivos candidatos, e semibold?)
#[cfg(windows)]
fn candidatas() -> Vec<(&'static str, Vec<std::path::PathBuf>, bool)> {
    let pasta = std::env::var_os("WINDIR").map_or_else(|| "C:\\Windows".into(), std::path::PathBuf::from).join("Fonts");
    let f = |nomes: &[&str]| nomes.iter().map(|n| pasta.join(n)).collect::<Vec<_>>();
    vec![
        ("segoe", f(&["segoeui.ttf"]), false),
        ("segoe-sb", f(&["seguisb.ttf", "segoeuib.ttf"]), true),
        ("simbolos", f(&["seguisym.ttf"]), false),
        ("ja", f(&["YuGothM.ttc", "meiryo.ttc", "msgothic.ttc"]), false),
        ("zh", f(&["msyh.ttc", "simsun.ttc"]), false),
        ("ko", f(&["malgun.ttf", "gulim.ttc"]), false),
    ]
}

#[cfg(not(windows))]
fn candidatas() -> Vec<(&'static str, Vec<std::path::PathBuf>, bool)> {
    let p = |s: &str| std::path::PathBuf::from(s);
    vec![
        ("inter", vec![p("/usr/share/fonts/opentype/inter/Inter-Regular.otf")], false),
        ("inter-sb", vec![p("/usr/share/fonts/opentype/inter/Inter-SemiBold.otf")], true),
        ("dejavu", vec![p("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf")], false),
        ("cjk", vec![p("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc")], false),
    ]
}

fn mapear(caminho: &std::path::Path) -> Option<FontData> {
    let arquivo = std::fs::File::open(caminho).ok()?;
    // SAFETY: fonte do sistema, somente leitura. Se alguem trocar o arquivo com o app aberto
    // o pior caso e glifo errado, nao corrupcao de memoria do processo.
    let mapa = unsafe { memmap2::Mmap::map(&arquivo) }.ok()?;
    Some(FontData::from_blob(Arc::new(mapa), 0))
}
