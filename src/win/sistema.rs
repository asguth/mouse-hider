//! Chamadas soltas do Win32: ociosidade, instancia unica, autostart, idioma, navegador.

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr::{null, null_mut};

use windows_sys::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError};
use windows_sys::Win32::Globalization::GetUserDefaultUILanguage;
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SZ, RRF_RT_REG_SZ, RegCloseKey, RegDeleteValueW,
    RegGetValueW, RegOpenKeyExW, RegSetValueExW,
};
use windows_sys::Win32::System::SystemInformation::GetTickCount;
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

use crate::nucleo::config::log;
use crate::nucleo::ocioso;

pub fn wide(texto: impl AsRef<OsStr>) -> Vec<u16> {
    texto.as_ref().encode_wide().chain(Some(0)).collect()
}

pub fn ticks() -> u32 {
    unsafe { GetTickCount() }
}

/// Tempo desde a ultima entrada de mouse ou teclado, em ms. Nao exige hook global: mais leve
/// e sem o falso positivo de keylogger em antivirus.
pub fn ocioso_ms() -> u32 {
    let mut info = LASTINPUTINFO { cbSize: size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
    if unsafe { GetLastInputInfo(&mut info) } == 0 {
        return 0;
    }
    ocioso::desde(ticks(), info.dwTime)
}

/// Duas instancias brigando pelo estado do cursor = cursor preso invisivel. O handle fica
/// aberto ate o processo sair, de proposito.
pub fn primeira_instancia() -> bool {
    let nome = wide("Local\\MouseHider");
    let handle = unsafe { CreateMutexW(null(), 1, nome.as_ptr()) };
    if handle.is_null() {
        return false;
    }
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        unsafe { CloseHandle(handle) };
        return false;
    }
    true
}

/// Idioma da interface do Windows em duas letras; o que nao tiver traducao cai no ingles.
pub fn idioma() -> &'static str {
    match unsafe { GetUserDefaultUILanguage() } & 0x3FF {
        0x16 => "pt",
        0x0A => "es",
        0x0C => "fr",
        0x07 => "de",
        0x10 => "it",
        0x19 => "ru",
        0x11 => "ja",
        0x04 => "zh",
        0x12 => "ko",
        _ => "en",
    }
}

pub fn abrir_url(url: &str) {
    let (verbo, alvo) = (wide("open"), wide(url));
    let r = unsafe { ShellExecuteW(null_mut(), verbo.as_ptr(), alvo.as_ptr(), null(), null(), SW_SHOWNORMAL) };
    if (r as isize) <= 32 {
        log(&format!("falha ao abrir {url} (codigo {})", r as isize));
    }
}

/// Roda um programa sem esperar (a versao nova, depois de atualizar).
pub fn executar(caminho: &std::path::Path, argumentos: &str) -> bool {
    let (verbo, alvo, args) = (wide("open"), wide(caminho), wide(argumentos));
    let r = unsafe { ShellExecuteW(null_mut(), verbo.as_ptr(), alvo.as_ptr(), args.as_ptr(), null(), SW_SHOWNORMAL) };
    let ok = (r as isize) > 32;
    if !ok {
        log(&format!("falha ao rodar {} (codigo {})", caminho.display(), r as isize));
    }
    ok
}

// --- autostart em HKCU\...\Run: so o usuario atual, sem administrador ---

const RUN: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const VALOR: &str = "MouseHider";

fn exe_entre_aspas() -> Option<String> {
    std::env::current_exe().ok().map(|p| format!("\"{}\"", p.display()))
}

fn abrir_run(acesso: u32) -> Option<HKEY> {
    let mut chave: HKEY = null_mut();
    let caminho = wide(RUN);
    let r = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, caminho.as_ptr(), 0, acesso, &mut chave) };
    (r == 0).then_some(chave)
}

fn ler_autostart() -> Option<String> {
    let chave = abrir_run(KEY_QUERY_VALUE)?;
    let nome = wide(VALOR);
    let mut buffer = vec![0u16; 1024];
    let mut bytes = (buffer.len() * 2) as u32;
    let r = unsafe {
        RegGetValueW(chave, null(), nome.as_ptr(), RRF_RT_REG_SZ, null_mut(), buffer.as_mut_ptr().cast(), &mut bytes)
    };
    unsafe { RegCloseKey(chave) };
    if r != 0 {
        return None;
    }
    let fim = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    Some(String::from_utf16_lossy(&buffer[..fim]))
}

fn gravar_autostart(valor: Option<&str>) {
    let Some(chave) = abrir_run(KEY_SET_VALUE) else {
        log("falha no autostart: chave Run inacessivel");
        return;
    };
    let nome = wide(VALOR);
    let r = match valor {
        Some(v) => {
            let dados = wide(v);
            unsafe { RegSetValueExW(chave, nome.as_ptr(), 0, REG_SZ, dados.as_ptr().cast(), (dados.len() * 2) as u32) }
        }
        None => unsafe { RegDeleteValueW(chave, nome.as_ptr()) },
    };
    unsafe { RegCloseKey(chave) };
    if r != 0 && valor.is_some() {
        log(&format!("falha no autostart (codigo {r})"));
    }
}

pub fn autostart() -> bool {
    ler_autostart().is_some()
}

pub fn definir_autostart(ligado: bool) {
    if ligado {
        if let Some(exe) = exe_entre_aspas() {
            gravar_autostart(Some(&exe));
        }
    } else {
        gravar_autostart(None);
    }
}

/// Autostart ligado mas apontando para outro caminho: reescreve para o exe atual. Sem isso,
/// mover ou reinstalar o app deixa o Windows tentando abrir um arquivo que nao existe mais.
pub fn corrigir_autostart() {
    let (Some(atual), Some(esperado)) = (ler_autostart(), exe_entre_aspas()) else { return };
    if !atual.eq_ignore_ascii_case(&esperado) {
        gravar_autostart(Some(&esperado));
        log(&format!("autostart corrigido: {atual} -> {esperado}"));
    }
}
