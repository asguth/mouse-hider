//! O app de verdade, no Windows: bandeja, atalho global, cursor do sistema e a janela.

mod bandeja;
mod cursor;
mod motor;
mod rede;
mod sinal;
mod sistema;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::{Arc, OnceLock};

use eframe::egui;
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::Graphics::Dwm::{DWMWA_CLOAK, DwmSetWindowAttribute};
use windows_sys::Win32::UI::Shell::{DefSubclassProc, SetWindowSubclass};
use windows_sys::Win32::UI::WindowsAndMessaging::{SC_MINIMIZE, SW_HIDE, ShowWindow, WM_SYSCOMMAND};

use crate::nucleo::config::{self, Config, log};
use crate::nucleo::plataforma::{ErroRede, Plataforma};
use crate::nucleo::{atalho, atualizacao, idiomas, logo};
use crate::ui::{self, Aba, Estado};

static ESTADO: OnceLock<Arc<Estado>> = OnceLock::new();
static CONTEXTO: OnceLock<egui::Context> = OnceLock::new();
static JANELA: AtomicIsize = AtomicIsize::new(0);

pub fn principal(args: &[String]) -> i32 {
    if args.iter().any(|a| a == "--selftest") {
        return autoteste();
    }

    // Versao nova aberta pela antiga durante a atualizacao: espera a antiga sair.
    let atualizado = args.iter().any(|a| a == "--atualizado");
    let tentativas = if atualizado { 50 } else { 1 };
    if !(0..tentativas).any(|i| {
        if i > 0 {
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        sistema::primeira_instancia()
    }) {
        sinal::avisar_instancia_existente();
        return 0;
    }
    if atualizado {
        apagar_exe_antigo();
        log(&format!("atualizado para {}", env!("CARGO_PKG_VERSION")));
    }

    // Antes de tudo: se a instancia anterior morreu com o cursor escondido, devolve o cursor.
    cursor::restaurar_se_ficou_escondido();

    // Todo caminho de saida restaura o cursor. E o requisito que manda no projeto.
    let anterior = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log(&format!("crash: {info}"));
        cursor::restaurar();
        anterior(info);
    }));

    let primeira_vez = Config::primeira_vez();
    let config = Config::carregar();
    if primeira_vez {
        config.salvar(); // senao a janela abriria sozinha em todo login ate mudar algo
    }
    idiomas::definir(&config.idioma, sistema::idioma());
    sistema::corrigir_autostart();
    permitir_menu_escuro();

    let estado = Arc::new(Estado::new(config.clone()));
    let _ = ESTADO.set(estado.clone());
    motor::iniciar(estado.clone());
    sinal::iniciar();

    let plataforma: Arc<dyn Plataforma> = Arc::new(Windows { estado: estado.clone() });
    plataforma.aplicar_atalho(&config.hotkey);
    log(&format!(
        "iniciado {} (ocioso={}s, pausado={}, atalho={}, idioma={})",
        env!("CARGO_PKG_VERSION"),
        config.idle_seconds,
        config.paused,
        config.hotkey,
        config.idioma
    ));

    let lado = 64;
    let opcoes = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Mouse Hider")
            .with_app_id("MouseHider")
            .with_inner_size([ui::LARGURA, ui::ALTURA])
            .with_resizable(false)
            .with_maximize_button(false)
            .with_icon(egui::IconData { rgba: logo::rgba(lado, false), width: lado, height: lado }),
        centered: true,
        ..Default::default()
    };

    let resultado = eframe::run_native(
        "Mouse Hider",
        opcoes,
        Box::new(move |cc| {
            let _ = CONTEXTO.set(cc.egui_ctx.clone());
            if let Ok(alca) = cc.window_handle()
                && let RawWindowHandle::Win32(w) = alca.as_raw()
            {
                let hwnd = w.hwnd.get();
                JANELA.store(hwnd, Ordering::SeqCst);
                preparar_janela(hwnd as HWND, !(primeira_vez || atualizado));
            }
            bandeja::criar(estado.clone());
            // Primeira execucao abre a janela: sem isso o app some na bandeja e parece que nao
            // abriu. Depois de atualizar tambem, para a pessoa ver que deu certo.
            let mostrar = primeira_vez || atualizado;
            Ok(Box::new(ui::App::new(&cc.egui_ctx, estado, plataforma, mostrar)))
        }),
    );
    if let Err(e) = resultado {
        log(&format!("falha na janela: {e}"));
    }

    // O X da janela encerra o app; o minimizar e que manda para a bandeja.
    bandeja::remover();
    cursor::restaurar();
    log("encerrado");
    0
}

/// Pede a janela e acorda o egui (ele nao roda sozinho com a janela oculta).
pub(crate) fn abrir_janela(aba: Aba) {
    if let Some(estado) = ESTADO.get() {
        estado.pedir_janela(aba);
    }
    if let Some(ctx) = CONTEXTO.get() {
        ctx.request_repaint();
    }
}

/// Depois de mudar `paused` no config: bandeja, cursor e janela acompanham.
pub(crate) fn pausa_alterada(estado: &Estado) {
    let pausado = estado.config().paused;
    motor::cancelar_manual();
    if pausado {
        cursor::restaurar();
    }
    bandeja::atualizar(estado);
    if let Some(ctx) = CONTEXTO.get() {
        ctx.request_repaint();
    }
    log(if pausado { "pausado" } else { "retomado" });
}

/// Saida pelo menu da bandeja ou depois de atualizar. Roda na thread da interface.
pub(crate) fn encerrar() -> ! {
    cursor::restaurar();
    bandeja::remover();
    log("encerrado");
    std::process::exit(0);
}

// --- atualizacao: troca do proprio exe ---

fn caminho_exe() -> Option<PathBuf> {
    std::env::current_exe().ok()
}

/// O Windows deixa renomear o exe em uso (so nao deixa sobrescrever). Entao: atual vira
/// .old, o baixado toma o nome do atual, a versao nova abre e esta sai.
fn trocar_exe(novo: &Path) -> bool {
    let Some(atual) = caminho_exe() else { return false };
    let antigo = atual.with_extension("exe.old");
    let _ = std::fs::remove_file(&antigo);
    if let Err(e) = std::fs::rename(&atual, &antigo) {
        log(&format!("falha ao renomear o exe atual: {e}"));
        return false;
    }
    if let Err(e) = std::fs::rename(novo, &atual) {
        log(&format!("falha ao colocar o exe novo no lugar: {e}"));
        let _ = std::fs::rename(&antigo, &atual);
        return false;
    }
    if sistema::executar(&atual, "--atualizado") {
        return true;
    }
    // Nao abriu: desfaz, para nao deixar a pessoa sem app nenhum.
    let _ = std::fs::rename(&atual, novo);
    let _ = std::fs::rename(&antigo, &atual);
    false
}

/// O .old so some depois que o processo antigo terminou de sair; tenta por alguns segundos.
fn apagar_exe_antigo() {
    let Some(antigo) = caminho_exe().map(|p| p.with_extension("exe.old")) else { return };
    std::thread::spawn(move || {
        for _ in 0..30 {
            if !antigo.exists() || std::fs::remove_file(&antigo).is_ok() {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });
}

// --- janela ---

/// DWM "cloak": a janela existe e o eframe a considera visivel, mas nada aparece. O eframe
/// mostra a janela depois do primeiro quadro; com o app abrindo escondido (autostart), isso
/// piscaria a janela na tela a cada login.
fn camuflar(hwnd: HWND, ligado: bool) {
    let valor: i32 = ligado.into();
    unsafe { DwmSetWindowAttribute(hwnd, DWMWA_CLOAK as u32, (&raw const valor).cast(), 4) };
}

fn preparar_janela(hwnd: HWND, abrir_oculta: bool) {
    if abrir_oculta {
        camuflar(hwnd, true);
    }
    unsafe { SetWindowSubclass(hwnd, Some(subclasse), 1, 0) };
}

/// Minimizar some na bandeja em vez de virar botao na barra de tarefas: o app ja vive la.
/// Intercepta o SC_MINIMIZE e nao repassa, entao a janela nunca fica minimizada e reabrir
/// e um simples mostrar.
unsafe extern "system" fn subclasse(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM, _: usize, _: usize) -> LRESULT {
    if msg == WM_SYSCOMMAND && (wparam & 0xFFF0) == SC_MINIMIZE as usize {
        unsafe { ShowWindow(hwnd, SW_HIDE) };
        return 0;
    }
    unsafe { DefSubclassProc(hwnd, msg, wparam, lparam) }
}

/// Menu de contexto da bandeja no tema escuro. Ordinais 135/136 do uxtheme
/// (SetPreferredAppMode/FlushMenuThemes): nao documentados, mas estaveis desde o Windows 10
/// 1903 e usados pelo proprio Explorer. Sem eles o menu fica claro com o sistema escuro.
fn permitir_menu_escuro() {
    use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};
    unsafe {
        let ux = LoadLibraryW(sistema::wide("uxtheme.dll").as_ptr());
        if ux.is_null() {
            return;
        }
        if let Some(definir) = GetProcAddress(ux, 135 as _) {
            let definir: unsafe extern "system" fn(i32) -> i32 = std::mem::transmute(definir);
            definir(1); // AllowDark: segue o tema do sistema
        }
        if let Some(atualizar) = GetProcAddress(ux, 136 as _) {
            let atualizar: unsafe extern "system" fn() = std::mem::transmute(atualizar);
            atualizar();
        }
    }
}

// --- plataforma ---

struct Windows {
    estado: Arc<Estado>,
}

impl Plataforma for Windows {
    fn ocioso_ms(&self) -> u32 {
        sistema::ocioso_ms()
    }

    fn cursor_escondido(&self) -> bool {
        cursor::escondido()
    }

    fn aplicar_atalho(&self, texto: &str) -> bool {
        let ok = atalho::interpretar(texto).is_some_and(|(mods, vk)| sinal::registrar(mods, vk));
        log(&if ok { format!("atalho registrado: {texto}") } else { format!("atalho recusado: {texto}") });
        ok
    }

    fn soltar_atalho(&self) {
        sinal::soltar();
    }

    fn autostart(&self) -> bool {
        sistema::autostart()
    }

    fn definir_autostart(&self, ligado: bool) {
        sistema::definir_autostart(ligado);
    }

    fn pausa_alterada(&self) {
        pausa_alterada(&self.estado);
    }

    fn idioma_alterado(&self) {
        bandeja::atualizar(&self.estado);
    }

    fn idioma_do_sistema(&self) -> &'static str {
        sistema::idioma()
    }

    fn abrir_url(&self, url: &str) {
        sistema::abrir_url(url);
    }

    fn buscar_release(&self, user_agent: &str) -> Result<String, ErroRede> {
        rede::buscar(atualizacao::API, user_agent)
    }

    /// Ao lado do exe atual: mesmo volume, entao a troca e um rename. Pasta sem escrita
    /// (o exe foi parar em Arquivos de Programas) = None, e a tela abre a pagina do release.
    fn destino_atualizacao(&self) -> Option<PathBuf> {
        let destino = caminho_exe()?.with_extension("exe.new");
        std::fs::File::create(&destino).ok()?;
        Some(destino)
    }

    fn baixar(&self, url: &str, user_agent: &str, destino: &Path, progresso: &mut dyn FnMut(u64, u64)) -> Result<(), String> {
        rede::baixar(url, user_agent, destino, progresso)
    }

    fn instalar(&self, novo: &Path) -> bool {
        trocar_exe(novo)
    }

    fn antes_de_mostrar(&self) {
        camuflar(JANELA.load(Ordering::SeqCst) as HWND, false);
    }

    fn sair(&self) {
        encerrar();
    }
}

// --- MouseHider.exe --selftest ---

/// Verificacao embutida: aritmetica, atalho e o ciclo esconder/restaurar no Windows real.
/// Sai com 0 ou 1; o release.ps1 roda isso antes de empacotar.
fn autoteste() -> i32 {
    use windows_sys::Win32::System::Console::{ATTACH_PARENT_PROCESS, AttachConsole};
    unsafe { AttachConsole(ATTACH_PARENT_PROCESS) }; // exe de janela nao tem console propria

    type Verificacao = (&'static str, Box<dyn Fn() -> bool>);
    let verificacoes: Vec<Verificacao> = vec![
        ("wraparound do contador de ticks", Box::new(|| crate::nucleo::ocioso::desde(i32::MIN as u32, i32::MAX as u32) == 1)),
        ("atalho padrao interpretado", Box::new(|| atalho::interpretar("Ctrl+Alt+H").is_some())),
        ("ociosidade legivel", Box::new(|| sistema::ocioso_ms() < u32::MAX)),
        ("esconder", Box::new(|| {
            cursor::esconder();
            cursor::escondido() && config::sentinela_existe()
        })),
        ("restaurar", Box::new(|| {
            cursor::restaurar();
            !cursor::escondido() && !config::sentinela_existe()
        })),
    ];

    for (nome, teste) in &verificacoes {
        if !teste() {
            cursor::restaurar(); // nunca deixar o cursor escondido por causa do teste
            eprintln!("selftest FALHOU: {nome}");
            return 1;
        }
    }
    cursor::restaurar();
    println!("selftest OK");
    0
}
