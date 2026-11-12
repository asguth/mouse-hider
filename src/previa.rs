//! Fora do Windows o app nao tem o que esconder: roda so a janela, com uma plataforma falsa,
//! para desenvolver a interface e tirar as screenshots do README.
//!
//!     cargo run -- [--aba geral|sistema|sobre] [--tema claro|escuro] [--idioma pt]

use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use eframe::egui;

use crate::nucleo::config::Config;
use crate::nucleo::idiomas;
use crate::nucleo::plataforma::{ErroRede, Plataforma};
use crate::ui::{self, Aba, Estado};

struct Falsa {
    inicio: Instant,
}

impl Plataforma for Falsa {
    fn ocioso_ms(&self) -> u32 {
        // Uma "mexida" a cada 4 s, para o contador andar.
        (self.inicio.elapsed().as_millis() % 4000) as u32 + 1200
    }
    fn cursor_escondido(&self) -> bool {
        false
    }
    fn aplicar_atalho(&self, _: &str) -> bool {
        true
    }
    fn soltar_atalho(&self) {}
    fn autostart(&self) -> bool {
        true
    }
    fn definir_autostart(&self, _: bool) {}
    fn pausa_alterada(&self) {}
    fn idioma_alterado(&self) {}
    fn idioma_do_sistema(&self) -> &'static str {
        "pt"
    }
    fn abrir_url(&self, url: &str) {
        println!("abrir {url}");
    }
    fn buscar_release(&self, _: &str) -> Result<String, ErroRede> {
        std::thread::sleep(std::time::Duration::from_millis(800));
        Ok(r#"{"tag_name": "v9.9.9", "assets": []}"#.into())
    }
    fn destino_atualizacao(&self) -> Option<std::path::PathBuf> {
        Some(std::env::temp_dir().join("MouseHider.exe.new"))
    }
    fn baixar(&self, _: &str, _: &str, _: &Path, _: &mut dyn FnMut(u64, u64)) -> Result<(), String> {
        Err("previa".into())
    }
    fn instalar(&self, _: &Path) -> bool {
        false
    }
    fn sair(&self) {
        std::process::exit(0);
    }
}

pub fn rodar(args: &[String]) {
    let valor = |nome: &str| args.iter().position(|a| a == nome).and_then(|i| args.get(i + 1)).cloned();
    let config = Config { idioma: valor("--idioma").unwrap_or_else(|| "pt".into()), ..Config::default() };
    idiomas::definir(&config.idioma, "pt");

    let aba = match valor("--aba").as_deref() {
        Some("sistema") => Aba::Sistema,
        Some("sobre") => Aba::Sobre,
        _ => Aba::Geral,
    };
    let tema = match valor("--tema").as_deref() {
        Some("escuro") => egui::ThemePreference::Dark,
        Some("claro") => egui::ThemePreference::Light,
        _ => egui::ThemePreference::System,
    };

    let estado = Arc::new(Estado::new(config));
    estado.pedir_janela(aba);
    let opcoes = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Mouse Hider")
            .with_inner_size([ui::LARGURA, ui::ALTURA])
            .with_resizable(false),
        ..Default::default()
    };
    let plataforma: Arc<dyn Plataforma> = Arc::new(Falsa { inicio: Instant::now() });
    let _ = eframe::run_native(
        "Mouse Hider",
        opcoes,
        Box::new(move |cc| {
            let app = ui::App::new(&cc.egui_ctx, estado, plataforma, true);
            cc.egui_ctx.set_theme(tema);
            Ok(Box::new(app))
        }),
    );
}
