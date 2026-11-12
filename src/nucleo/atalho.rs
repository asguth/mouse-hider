//! Atalho global como texto ("Ctrl+Alt+H"), no mesmo formato da versao .NET, e a conversao
//! para o que o RegisterHotKey quer (modificadores + virtual key).

use eframe::egui::{Key, Modifiers};

pub const MOD_ALT: u32 = 0x0001;
pub const MOD_CONTROL: u32 = 0x0002;
pub const MOD_SHIFT: u32 = 0x0004;
pub const MOD_NOREPEAT: u32 = 0x4000;

/// Nome gravado no config, virtual key e a tecla do egui. O nome segue o enum Keys do
/// WinForms, para a config antiga continuar valendo.
const TECLAS: &[(&str, u32, Key)] = &[
    ("A", 0x41, Key::A), ("B", 0x42, Key::B), ("C", 0x43, Key::C), ("D", 0x44, Key::D),
    ("E", 0x45, Key::E), ("F", 0x46, Key::F), ("G", 0x47, Key::G), ("H", 0x48, Key::H),
    ("I", 0x49, Key::I), ("J", 0x4A, Key::J), ("K", 0x4B, Key::K), ("L", 0x4C, Key::L),
    ("M", 0x4D, Key::M), ("N", 0x4E, Key::N), ("O", 0x4F, Key::O), ("P", 0x50, Key::P),
    ("Q", 0x51, Key::Q), ("R", 0x52, Key::R), ("S", 0x53, Key::S), ("T", 0x54, Key::T),
    ("U", 0x55, Key::U), ("V", 0x56, Key::V), ("W", 0x57, Key::W), ("X", 0x58, Key::X),
    ("Y", 0x59, Key::Y), ("Z", 0x5A, Key::Z),
    ("0", 0x30, Key::Num0), ("1", 0x31, Key::Num1), ("2", 0x32, Key::Num2), ("3", 0x33, Key::Num3),
    ("4", 0x34, Key::Num4), ("5", 0x35, Key::Num5), ("6", 0x36, Key::Num6), ("7", 0x37, Key::Num7),
    ("8", 0x38, Key::Num8), ("9", 0x39, Key::Num9),
    ("F1", 0x70, Key::F1), ("F2", 0x71, Key::F2), ("F3", 0x72, Key::F3), ("F4", 0x73, Key::F4),
    ("F5", 0x74, Key::F5), ("F6", 0x75, Key::F6), ("F7", 0x76, Key::F7), ("F8", 0x77, Key::F8),
    ("F9", 0x78, Key::F9), ("F10", 0x79, Key::F10), ("F11", 0x7A, Key::F11), ("F12", 0x7B, Key::F12),
    ("F13", 0x7C, Key::F13), ("F14", 0x7D, Key::F14), ("F15", 0x7E, Key::F15), ("F16", 0x7F, Key::F16),
    ("F17", 0x80, Key::F17), ("F18", 0x81, Key::F18), ("F19", 0x82, Key::F19), ("F20", 0x83, Key::F20),
    ("Space", 0x20, Key::Space), ("Enter", 0x0D, Key::Enter), ("Tab", 0x09, Key::Tab),
    ("Back", 0x08, Key::Backspace), ("Insert", 0x2D, Key::Insert), ("Delete", 0x2E, Key::Delete),
    ("Home", 0x24, Key::Home), ("End", 0x23, Key::End), ("PageUp", 0x21, Key::PageUp),
    ("PageDown", 0x22, Key::PageDown), ("Left", 0x25, Key::ArrowLeft), ("Up", 0x26, Key::ArrowUp),
    ("Right", 0x27, Key::ArrowRight), ("Down", 0x28, Key::ArrowDown),
    ("OemMinus", 0xBD, Key::Minus), ("Oemplus", 0xBB, Key::Equals), ("Oemcomma", 0xBC, Key::Comma),
    ("OemPeriod", 0xBE, Key::Period), ("OemQuestion", 0xBF, Key::Slash), ("OemSemicolon", 0xBA, Key::Semicolon),
    ("Oemtilde", 0xC0, Key::Backtick), ("OemOpenBrackets", 0xDB, Key::OpenBracket),
    ("OemCloseBrackets", 0xDD, Key::CloseBracket), ("OemPipe", 0xDC, Key::Backslash), ("OemQuotes", 0xDE, Key::Quote),
];

/// Sinonimos que o WinForms tambem aceitava.
const SINONIMOS: &[(&str, &str)] = &[("Return", "Enter"), ("Prior", "PageUp"), ("Next", "PageDown"), ("Backspace", "Back")];

/// Rotulo curto para a tecla na tela (o nome do WinForms e feio para "OemMinus").
pub fn rotulo(nome: &str) -> &str {
    match nome {
        "OemMinus" => "-",
        "Oemplus" => "=",
        "Oemcomma" => ",",
        "OemPeriod" => ".",
        "OemQuestion" => "/",
        "OemSemicolon" => ";",
        "Oemtilde" => "`",
        "OemOpenBrackets" => "[",
        "OemCloseBrackets" => "]",
        "OemPipe" => "\\",
        "OemQuotes" => "'",
        "Back" => "Backspace",
        "Left" => "←",
        "Up" => "↑",
        "Right" => "→",
        "Down" => "↓",
        outro => outro,
    }
}

/// Texto do atalho, ou None sem ao menos um modificador (atalho global sem Ctrl/Alt/Shift
/// roubaria a tecla do sistema inteiro).
pub fn formatar(tecla: Key, mods: Modifiers) -> Option<String> {
    let (nome, _, _) = TECLAS.iter().find(|(_, _, k)| *k == tecla)?;
    let mut partes = Vec::new();
    if mods.ctrl {
        partes.push("Ctrl");
    }
    if mods.alt {
        partes.push("Alt");
    }
    if mods.shift {
        partes.push("Shift");
    }
    if partes.is_empty() {
        return None;
    }
    partes.push(nome);
    Some(partes.join("+"))
}

/// (modificadores, virtual key) para o RegisterHotKey, ou None se o texto nao servir.
pub fn interpretar(texto: &str) -> Option<(u32, u32)> {
    let mut mods = 0;
    let mut vk = None;
    for parte in texto.split('+').map(str::trim).filter(|p| !p.is_empty()) {
        match parte.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => mods |= MOD_CONTROL,
            "alt" => mods |= MOD_ALT,
            "shift" => mods |= MOD_SHIFT,
            _ => vk = Some(vk_de(parte)?),
        }
    }
    if mods == 0 {
        return None;
    }
    Some((mods | MOD_NOREPEAT, vk?))
}

/// Partes para desenhar como teclas: ["Ctrl", "Alt", "H"].
pub fn partes(texto: &str) -> Vec<String> {
    texto.split('+').map(str::trim).filter(|p| !p.is_empty()).map(|p| rotulo(p).to_owned()).collect()
}

fn vk_de(nome: &str) -> Option<u32> {
    // "D5" e "NumPad5" sao os nomes do WinForms para os digitos.
    let nome = SINONIMOS.iter().find(|(s, _)| s.eq_ignore_ascii_case(nome)).map_or(nome, |(_, n)| *n);
    let digito = nome.strip_prefix(['D', 'd']).filter(|r| r.len() == 1 && r.as_bytes()[0].is_ascii_digit());
    if let Some(d) = digito {
        return Some(0x30 + u32::from(d.as_bytes()[0] - b'0'));
    }
    let numpad = nome.to_ascii_lowercase();
    if let Some(d) = numpad.strip_prefix("numpad").or_else(|| numpad.strip_prefix("num"))
        && let [c] = d.as_bytes()
        && c.is_ascii_digit()
    {
        return Some(0x60 + u32::from(c - b'0'));
    }
    TECLAS.iter().find(|(n, _, _)| n.eq_ignore_ascii_case(nome)).map(|(_, vk, _)| *vk)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn padrao_e_formatos_antigos() {
        assert_eq!(interpretar("Ctrl+Alt+H"), Some((MOD_CONTROL | MOD_ALT | MOD_NOREPEAT, 0x48)));
        assert_eq!(interpretar("Ctrl+Shift+D5"), Some((MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, 0x35)));
        assert_eq!(interpretar("Alt+5"), Some((MOD_ALT | MOD_NOREPEAT, 0x35)));
        assert_eq!(interpretar("Ctrl+Num3"), Some((MOD_CONTROL | MOD_NOREPEAT, 0x63)));
        assert_eq!(interpretar("ctrl + f9"), Some((MOD_CONTROL | MOD_NOREPEAT, 0x78)));
        assert_eq!(interpretar("Alt+Return"), Some((MOD_ALT | MOD_NOREPEAT, 0x0D)));
        assert_eq!(interpretar("Alt+Delete"), Some((MOD_ALT | MOD_NOREPEAT, 0x2E)));
    }

    #[test]
    fn recusa_sem_modificador_ou_sem_tecla() {
        assert_eq!(interpretar("H"), None);
        assert_eq!(interpretar("Ctrl+Alt"), None);
        assert_eq!(interpretar("Ctrl+Banana"), None);
        assert_eq!(interpretar(""), None);
    }

    #[test]
    fn ida_e_volta() {
        let mods = Modifiers { ctrl: true, shift: true, ..Default::default() };
        let texto = formatar(Key::Minus, mods).unwrap();
        assert_eq!(texto, "Ctrl+Shift+OemMinus");
        assert_eq!(interpretar(&texto), Some((MOD_CONTROL | MOD_SHIFT | MOD_NOREPEAT, 0xBD)));
        assert_eq!(partes(&texto), ["Ctrl", "Shift", "-"]);
        assert_eq!(formatar(Key::H, Modifiers::NONE), None);
    }
}
