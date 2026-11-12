//! Icone e menu da bandeja. O menu espelha as abas da janela: clicar leva direto na aba certa.
//!
//! TrayIcon e os itens do menu nao sao Send: vivem num thread_local da thread da interface,
//! que e onde o Windows entrega os eventos deles.

use std::cell::RefCell;
use std::sync::Arc;

use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSMICON};

use crate::nucleo::config::log;
use crate::nucleo::idiomas::{T, t};
use crate::nucleo::logo;
use crate::ui::{Aba, Estado};

struct Bandeja {
    icone: TrayIcon,
    geral: MenuItem,
    sistema: MenuItem,
    sobre: MenuItem,
    pausar: CheckMenuItem,
    sair: MenuItem,
}

thread_local! {
    static BANDEJA: RefCell<Option<Bandeja>> = const { RefCell::new(None) };
}

fn icone(pausado: bool) -> Option<Icon> {
    let lado = unsafe { GetSystemMetrics(SM_CXSMICON) }.clamp(16, 64) as u32;
    Icon::from_rgba(logo::rgba(lado, pausado), lado, lado).ok()
}

fn dica(pausado: bool) -> String {
    if pausado { format!("Mouse Hider · {}", t(T::Pausado)) } else { "Mouse Hider".into() }
}

pub fn criar(estado: Arc<Estado>) {
    let pausado = estado.config().paused;
    let geral = MenuItem::with_id("geral", t(T::Geral), true, None);
    let sistema = MenuItem::with_id("sistema", t(T::Sistema), true, None);
    let sobre = MenuItem::with_id("sobre", t(T::Sobre), true, None);
    let pausar = CheckMenuItem::with_id("pausar", t(T::MenuPausar), true, pausado, None);
    let sair = MenuItem::with_id("sair", t(T::MenuSair), true, None);

    let menu = Menu::new();
    let separador1 = PredefinedMenuItem::separator();
    let separador2 = PredefinedMenuItem::separator();
    if let Err(e) = menu.append_items(&[&geral, &sistema, &sobre, &separador1, &pausar, &separador2, &sair]) {
        log(&format!("falha ao montar o menu da bandeja: {e}"));
    }

    let mut construtor = TrayIconBuilder::new()
        .with_id("mouse-hider")
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .with_tooltip(dica(pausado));
    if let Some(i) = icone(pausado) {
        construtor = construtor.with_icon(i);
    }
    let icone = match construtor.build() {
        Ok(i) => i,
        Err(e) => {
            log(&format!("falha ao criar o icone da bandeja: {e}"));
            return;
        }
    };

    let estado_menu = estado.clone();
    MenuEvent::set_event_handler(Some(move |evento: MenuEvent| match evento.id.as_ref() {
        "geral" => super::abrir_janela(Aba::Geral),
        "sistema" => super::abrir_janela(Aba::Sistema),
        "sobre" => super::abrir_janela(Aba::Sobre),
        "pausar" => {
            {
                let mut c = estado_menu.config();
                c.paused = !c.paused;
                c.salvar();
            }
            super::pausa_alterada(&estado_menu);
        }
        "sair" => super::encerrar(),
        _ => {}
    }));

    // Clique simples abre a janela, como nos apps de bandeja do proprio Windows.
    TrayIconEvent::set_event_handler(Some(|evento: TrayIconEvent| match evento {
        TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. }
        | TrayIconEvent::DoubleClick { button: MouseButton::Left, .. } => super::abrir_janela(Aba::Geral),
        _ => {}
    }));

    BANDEJA.with(|b| *b.borrow_mut() = Some(Bandeja { icone, geral, sistema, sobre, pausar, sair }));
}

/// Reaplica textos, marca de pausa, icone e dica. Chamar na thread da interface.
pub fn atualizar(estado: &Estado) {
    let pausado = estado.config().paused;
    BANDEJA.with(|b| {
        if let Some(b) = b.borrow().as_ref() {
            b.geral.set_text(t(T::Geral));
            b.sistema.set_text(t(T::Sistema));
            b.sobre.set_text(t(T::Sobre));
            b.pausar.set_text(t(T::MenuPausar));
            b.pausar.set_checked(pausado);
            b.sair.set_text(t(T::MenuSair));
            let _ = b.icone.set_icon(icone(pausado)); // pausado = logo em cinza
            let _ = b.icone.set_tooltip(Some(dica(pausado)));
        }
    });
}

/// Tira o icone da bandeja agora; sem isso ele fica la ate alguem passar o mouse em cima.
pub fn remover() {
    let _ = BANDEJA.try_with(|b| b.borrow_mut().take());
}
