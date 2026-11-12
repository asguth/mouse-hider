//! Laco de 50 ms numa thread propria. Esse intervalo e a latencia com que o cursor volta no
//! primeiro movimento; com 500 ms a demora era visivel. O tique e so um GetLastInputInfo.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::nucleo::config::log;
use crate::nucleo::ocioso::{self, Acao};
use crate::ui::Estado;

use super::{cursor, sistema};

/// Marca (em ticks) do esconder pelo atalho; None = nao ha esconder manual valendo.
static MANUAL: Mutex<Option<u32>> = Mutex::new(None);

fn manual() -> std::sync::MutexGuard<'static, Option<u32>> {
    MANUAL.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn iniciar(estado: Arc<Estado>) {
    std::thread::Builder::new()
        .name("motor".into())
        .spawn(move || loop {
            tique(&estado);
            std::thread::sleep(Duration::from_millis(50));
        })
        .expect("thread do motor");
}

fn tique(estado: &Estado) {
    let agora = sistema::ticks();
    let ocioso_ms = sistema::ocioso_ms();
    let (pausado, limite) = {
        let c = estado.config();
        (c.paused, c.idle_seconds)
    };

    let manual_ativo = {
        let mut marca = manual();
        if marca.is_some_and(|m| ocioso::manual_expirou(agora, ocioso_ms, m)) {
            *marca = None;
        }
        marca.is_some()
    };

    match ocioso::decidir(pausado, manual_ativo, ocioso_ms, limite) {
        Acao::Esconder => cursor::esconder(),
        // So restaura quando ha o que restaurar: restaurar consulta a sentinela em disco, e a
        // 50 ms isso viraria vinte leituras de disco por segundo.
        Acao::Mostrar if cursor::escondido() => cursor::restaurar(),
        Acao::Mostrar => {}
    }
}

/// Atalho global: esconde na hora, sem esperar a contagem.
pub fn esconder_agora(estado: &Estado) {
    if estado.config().paused {
        return;
    }
    *manual() = Some(sistema::ticks());
    cursor::esconder();
    log("escondido pelo atalho");
}

pub fn cancelar_manual() {
    *manual() = None;
}
