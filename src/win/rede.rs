//! HTTP pelo ureq com o TLS do proprio Windows (SChannel): sem OpenSSL, sem certificados
//! embutidos, e o proxy/antivirus corporativo que confia na loja do Windows continua valendo.

use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Duration;

use ureq::Agent;
use ureq::tls::{TlsConfig, TlsProvider};

use crate::nucleo::plataforma::ErroRede;

fn agente(tempo: Duration) -> Agent {
    Agent::config_builder()
        .tls_config(TlsConfig::builder().provider(TlsProvider::NativeTls).build())
        .timeout_global(Some(tempo))
        .build()
        .into()
}

fn classificar(e: ureq::Error) -> ErroRede {
    match e {
        ureq::Error::StatusCode(_) | ureq::Error::BadUri(_) | ureq::Error::Http(_) | ureq::Error::Protocol(_) => {
            ErroRede::Outro(e.to_string())
        }
        // Timeout, DNS, conexao recusada, TLS: tudo cara de "sem internet" para quem usa.
        outro => ErroRede::SemConexao(outro.to_string()),
    }
}

pub fn buscar(url: &str, user_agent: &str) -> Result<String, ErroRede> {
    let mut resposta = agente(Duration::from_secs(10))
        .get(url)
        .header("User-Agent", user_agent)
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(classificar)?;
    resposta.body_mut().read_to_string().map_err(classificar)
}

pub fn baixar(url: &str, user_agent: &str, destino: &Path, progresso: &mut dyn FnMut(u64, u64)) -> Result<(), String> {
    // O instalador passa de alguns MB; 10 minutos cobre conexao lenta.
    let resposta = agente(Duration::from_secs(600))
        .get(url)
        .header("User-Agent", user_agent)
        .call()
        .map_err(|e| e.to_string())?;
    let total = resposta.body().content_length().unwrap_or(0);
    let mut leitor = resposta.into_body().into_reader();
    let mut arquivo = File::create(destino).map_err(|e| e.to_string())?;

    let mut buffer = vec![0u8; 64 * 1024];
    let mut lidos = 0u64;
    loop {
        let n = leitor.read(&mut buffer).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        arquivo.write_all(&buffer[..n]).map_err(|e| e.to_string())?;
        lidos += n as u64;
        progresso(lidos, total);
    }
    arquivo.flush().map_err(|e| e.to_string())?;
    if total > 0 && lidos != total {
        return Err(format!("download truncado: {lidos} de {total} bytes"));
    }
    Ok(())
}
