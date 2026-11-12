//! Esconde e restaura os cursores do sistema inteiro (vale para todos os processos).
//! ShowCursor() nao serve: e por thread e nao afeta outras janelas.

use std::ptr::null_mut;
use std::sync::Mutex;

use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateCursor, SPI_SETCURSORS, SPIF_SENDCHANGE, SetSystemCursor, SystemParametersInfoW,
};

use crate::nucleo::config::{definir_sentinela, log, sentinela_existe};

/// Lista fixa dos OCR_* padrao. Cursores proprios de apps continuam visiveis: nao ha API
/// publica para enumerar cursores de terceiros.
const CURSORES: [u32; 13] = [
    32512, // OCR_NORMAL
    32513, 32514, 32515, 32516, // IBEAM, WAIT, CROSS, UP
    32642, 32643, 32644, 32645, 32646, // SIZENWSE, SIZENESW, SIZEWE, SIZENS, SIZEALL
    32648, 32649, 32650, // NO, HAND, APPSTARTING
];

static ESCONDIDO: Mutex<bool> = Mutex::new(false);

fn trava() -> std::sync::MutexGuard<'static, bool> {
    ESCONDIDO.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn escondido() -> bool {
    *trava()
}

pub fn esconder() {
    let mut escondido = trava();
    if *escondido {
        return;
    }

    // AND=1 + XOR=0 em cada pixel: cursor totalmente transparente.
    let and = [0xFFu8; 32 * 32 / 8];
    let xor = [0u8; 32 * 32 / 8];
    let mut trocados = 0;
    for id in CURSORES {
        // Um handle novo por id: SetSystemCursor assume a posse e destroi o handle.
        let h = unsafe { CreateCursor(null_mut(), 0, 0, 32, 32, and.as_ptr().cast(), xor.as_ptr().cast()) };
        if h.is_null() {
            continue;
        }
        if unsafe { SetSystemCursor(h, id) } != 0 {
            trocados += 1;
        }
    }
    if trocados == 0 {
        log("nenhum cursor pode ser trocado, desistindo de esconder");
        return;
    }
    *escondido = true;
    definir_sentinela(true);
}

/// Recarrega os cursores do registro. Idempotente e seguro de chamar de qualquer thread.
pub fn restaurar() {
    let mut escondido = trava();
    if !*escondido && !sentinela_existe() {
        return;
    }
    unsafe { SystemParametersInfoW(SPI_SETCURSORS, 0, null_mut(), SPIF_SENDCHANGE) };
    *escondido = false;
    definir_sentinela(false);
}

/// A instancia anterior morreu com o cursor escondido: restaura antes de qualquer outra coisa.
pub fn restaurar_se_ficou_escondido() {
    if sentinela_existe() {
        log("sentinela encontrada na abertura, restaurando o cursor da sessao anterior");
        restaurar();
    }
}
