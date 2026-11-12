//! `%AppData%\MouseHider\config.json`, a sentinela de cursor escondido e o log.
//!
//! Os nomes dos campos sao os da versao .NET (PascalCase): quem atualiza mantem a config.

use std::fs;
use std::io::Write;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub const OCIOSO_MIN: u32 = 1;
pub const OCIOSO_MAX: u32 = 3600;
const LOG_MAX_BYTES: u64 = 256 * 1024;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase", default)]
pub struct Config {
    pub idle_seconds: u32,
    pub paused: bool,
    pub hotkey: String,
    pub idioma: String,
}

impl Default for Config {
    fn default() -> Self {
        Self { idle_seconds: 5, paused: false, hotkey: "Ctrl+Alt+H".into(), idioma: "auto".into() }
    }
}

impl Config {
    /// Config ilegivel cai no padrao em silencio: nao vale uma tela de erro num app que roda
    /// invisivel. O motivo fica no log.
    pub fn carregar() -> Self {
        let mut config = match fs::read_to_string(caminho_config()) {
            Ok(texto) => serde_json::from_str::<Config>(texto.trim_start_matches('\u{feff}')).unwrap_or_else(|e| {
                log(&format!("config ilegivel, usando padrao: {e}"));
                Config::default()
            }),
            Err(_) => Config::default(),
        };
        config.idle_seconds = config.idle_seconds.clamp(OCIOSO_MIN, OCIOSO_MAX);
        config
    }

    pub fn salvar(&self) {
        let resultado = fs::create_dir_all(pasta())
            .and_then(|_| fs::write(caminho_config(), serde_json::to_string_pretty(self).unwrap_or_default()));
        if let Err(e) = resultado {
            log(&format!("falha ao salvar config: {e}"));
        }
    }

    /// Primeira execucao: ainda nao ha config gravada.
    pub fn primeira_vez() -> bool {
        !caminho_config().exists()
    }
}

#[cfg(windows)]
pub fn pasta() -> PathBuf {
    std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir).join("MouseHider")
}

/// Fora do Windows so roda a previa da interface; nada de mexer na home de ninguem.
#[cfg(not(windows))]
pub fn pasta() -> PathBuf {
    std::env::temp_dir().join("MouseHider-previa")
}

fn caminho_config() -> PathBuf {
    pasta().join("config.json")
}

fn caminho_sentinela() -> PathBuf {
    pasta().join("cursor-hidden.flag")
}

// --- sentinela de crash: existe enquanto o cursor esta escondido ---

pub fn sentinela_existe() -> bool {
    caminho_sentinela().exists()
}

pub fn definir_sentinela(ligada: bool) {
    let resultado = if ligada {
        fs::create_dir_all(pasta()).and_then(|_| fs::write(caminho_sentinela(), agora()))
    } else if sentinela_existe() {
        fs::remove_file(caminho_sentinela())
    } else {
        Ok(())
    };
    if let Err(e) = resultado {
        log(&format!("falha na sentinela: {e}"));
    }
}

// --- log ---

/// "Rotacao" = apagar ao passar de 256 KB: log de diagnostico, nao de auditoria.
/// UTF-8 com BOM no arquivo novo: sem ele o Bloco de Notas le acento como ANSI.
pub fn log(mensagem: &str) {
    let _ = (|| -> std::io::Result<()> {
        fs::create_dir_all(pasta())?;
        let caminho = pasta().join("log.txt");
        if fs::metadata(&caminho).is_ok_and(|m| m.len() > LOG_MAX_BYTES) {
            fs::remove_file(&caminho)?;
        }
        let novo = !caminho.exists();
        let mut arquivo = fs::OpenOptions::new().create(true).append(true).open(&caminho)?;
        if novo {
            arquivo.write_all(b"\xEF\xBB\xBF")?;
        }
        writeln!(arquivo, "{}  {mensagem}\r", agora())
    })();
}

/// Hora local "aaaa-mm-dd hh:mm:ss".
#[cfg(windows)]
fn agora() -> String {
    let mut t = unsafe { std::mem::zeroed::<windows_sys::Win32::Foundation::SYSTEMTIME>() };
    unsafe { windows_sys::Win32::System::SystemInformation::GetLocalTime(&mut t) };
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", t.wYear, t.wMonth, t.wDay, t.wHour, t.wMinute, t.wSecond)
}

#[cfg(not(windows))]
fn agora() -> String {
    let s = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    format!("{s} (UTC)")
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn le_a_config_da_versao_dotnet() {
        let antiga = "\u{feff}{\n  \"IdleSeconds\": 30,\n  \"Paused\": true,\n  \"Hotkey\": \"Ctrl+Shift+F9\",\n  \"Idioma\": \"pt\"\n}";
        let c: Config = serde_json::from_str(antiga.trim_start_matches('\u{feff}')).unwrap();
        assert_eq!(c, Config { idle_seconds: 30, paused: true, hotkey: "Ctrl+Shift+F9".into(), idioma: "pt".into() });
    }

    #[test]
    fn campo_faltando_usa_o_padrao() {
        let c: Config = serde_json::from_str("{\"IdleSeconds\": 12}").unwrap();
        assert_eq!(c.idle_seconds, 12);
        assert_eq!(c.hotkey, "Ctrl+Alt+H");
    }

    #[test]
    fn grava_o_mais_sem_escapar() {
        let texto = serde_json::to_string_pretty(&Config::default()).unwrap();
        assert!(texto.contains("\"Ctrl+Alt+H\""));
        assert!(texto.contains("\"IdleSeconds\": 5"));
    }
}
