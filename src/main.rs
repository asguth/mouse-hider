//! Mouse Hider: esconde o cursor depois de alguns segundos sem uso e o traz de volta no
//! primeiro movimento.

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]
// Fora do Windows so a previa da tela compila; o resto do nucleo fica sem uso.
#![cfg_attr(not(windows), allow(dead_code))]

mod nucleo;
mod ui;

#[cfg(windows)]
mod win;

#[cfg(not(windows))]
mod previa;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Gera assets/MouseHider.ico (o icone do exe). Roda de novo so quando o logo mudar.
    if let Some(i) = args.iter().position(|a| a == "--makeicon") {
        let destino = args.get(i + 1).map_or("assets/MouseHider.ico", String::as_str);
        if let Err(e) = std::fs::write(destino, nucleo::logo::ico()) {
            eprintln!("nao deu para gravar {destino}: {e}");
            std::process::exit(1);
        }
        return;
    }

    #[cfg(windows)]
    std::process::exit(win::principal(&args));

    #[cfg(not(windows))]
    previa::rodar(&args);
}
