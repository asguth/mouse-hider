//! Janela oculta numa thread propria, para o que chega do Windows sem passar pela interface:
//! o atalho global (WM_HOTKEY), o fim de sessao, o bloqueio de tela e o aviso de uma segunda
//! instancia. Fica fora da thread do egui para o atalho funcionar com a janela fechada.

use std::ptr::{null, null_mut};
use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::mpsc::channel;

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::RemoteDesktop::{NOTIFY_FOR_THIS_SESSION, WTSRegisterSessionNotification};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{RegisterHotKey, UnregisterHotKey};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    ASFW_ANY, AllowSetForegroundWindow, CreateWindowExW, DefWindowProcW, DispatchMessageW, FindWindowW, GetMessageW, MSG,
    PostMessageW, RegisterClassW, SendMessageW, TranslateMessage, WM_APP, WM_ENDSESSION, WM_HOTKEY, WM_QUERYENDSESSION,
    WM_WTSSESSION_CHANGE, WNDCLASSW, WS_EX_TOOLWINDOW, WS_POPUP,
};

use crate::nucleo::config::log;
use crate::ui::Aba;

use super::{cursor, motor, sistema::wide};

const CLASSE: &str = "MouseHiderSinal";
const WM_REGISTRAR: u32 = WM_APP + 1;
const WM_ABRIR: u32 = WM_APP + 2;
const ID_ATALHO: i32 = 1;

static JANELA: AtomicIsize = AtomicIsize::new(0);

fn janela() -> HWND {
    JANELA.load(Ordering::SeqCst) as HWND
}

/// Cria a janela e espera ela existir: o registro do atalho vem logo em seguida.
pub fn iniciar() {
    let (tx, rx) = channel();
    std::thread::Builder::new()
        .name("sinal".into())
        .spawn(move || unsafe {
            let instancia = GetModuleHandleW(null());
            let classe = wide(CLASSE);
            let wc = WNDCLASSW { lpfnWndProc: Some(procedimento), hInstance: instancia, lpszClassName: classe.as_ptr(), ..std::mem::zeroed() };
            RegisterClassW(&wc);
            let hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW,
                classe.as_ptr(),
                classe.as_ptr(),
                WS_POPUP,
                0,
                0,
                0,
                0,
                null_mut(),
                null_mut(),
                instancia,
                null(),
            );
            JANELA.store(hwnd as isize, Ordering::SeqCst);
            // Bloquear a tela ou trocar de usuario devolve o cursor.
            WTSRegisterSessionNotification(hwnd, NOTIFY_FOR_THIS_SESSION);
            let _ = tx.send(());

            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        })
        .expect("thread de sinais");
    let _ = rx.recv();
}

/// Troca o atalho ativo. Se a combinacao nova falhar, fica sem atalho e devolve false.
/// SendMessage entre threads e sincrono: a resposta e a do proprio RegisterHotKey.
pub fn registrar(mods: u32, vk: u32) -> bool {
    unsafe { SendMessageW(janela(), WM_REGISTRAR, mods as WPARAM, vk as LPARAM) != 0 }
}

pub fn soltar() {
    unsafe { SendMessageW(janela(), WM_REGISTRAR, 0, 0) };
}

/// Segunda instancia: pede para a primeira abrir a janela em vez de abrir outra copia.
pub fn avisar_instancia_existente() {
    let classe = wide(CLASSE);
    unsafe {
        let hwnd = FindWindowW(classe.as_ptr(), null());
        if !hwnd.is_null() {
            // Quem foi aberto pelo usuario pode ceder o primeiro plano para a outra instancia.
            AllowSetForegroundWindow(ASFW_ANY);
            PostMessageW(hwnd, WM_ABRIR, 0, 0);
        }
    }
}

unsafe extern "system" fn procedimento(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_HOTKEY => {
            if let Some(estado) = super::ESTADO.get() {
                motor::esconder_agora(estado);
            }
            0
        }
        WM_REGISTRAR => unsafe {
            UnregisterHotKey(hwnd, ID_ATALHO);
            if lparam == 0 {
                return 0;
            }
            (RegisterHotKey(hwnd, ID_ATALHO, wparam as u32, lparam as u32) != 0) as LRESULT
        },
        WM_ABRIR => {
            super::abrir_janela(Aba::Geral);
            0
        }
        WM_WTSSESSION_CHANGE => {
            cursor::restaurar();
            0
        }
        WM_QUERYENDSESSION => {
            // Desligar, reiniciar ou sair da conta: restaura antes de concordar em sair.
            cursor::restaurar();
            1
        }
        WM_ENDSESSION => {
            if wparam != 0 {
                cursor::restaurar();
                log("fim de sessao");
                std::process::exit(0);
            }
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
