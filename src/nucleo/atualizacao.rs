//! Consulta o release mais recente no GitHub e, havendo versao nova, baixa o exe e troca o
//! atual por ele. Sem instalador, sem servico de update e sem verificacao em segundo plano:
//! so acontece quando o usuario clica.

use std::fs;
use std::io::Read;
use std::path::PathBuf;

use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::config::log;
use super::plataforma::{ErroRede, Plataforma};
use super::versao;

pub const API: &str = "https://api.github.com/repos/asguth/mouse-hider/releases/latest";
pub const PAGINA: &str = "https://github.com/asguth/mouse-hider/releases/latest";
pub const REPOSITORIO: &str = "https://github.com/asguth/mouse-hider";
/// Nome fixo: e ele que sustenta o link permanente do README.
const ASSET: &str = "MouseHider.exe";

#[derive(Clone, Debug, PartialEq)]
pub struct Novidade {
    pub versao: String,
    /// None = release sem o exe anexado; a tela cai na pagina do release.
    pub url: Option<String>,
    pub sha256: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum Resultado {
    Atualizado,
    TemNova(Novidade),
    SemConexao,
    Falhou,
}

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    #[serde(default)]
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: Option<String>,
    digest: Option<String>,
}

pub fn user_agent() -> String {
    // A API do GitHub recusa requisicao sem User-Agent.
    format!("MouseHider/{}", env!("CARGO_PKG_VERSION"))
}

pub fn verificar(plataforma: &dyn Plataforma) -> Resultado {
    match plataforma.buscar_release(&user_agent()) {
        Ok(corpo) => interpretar(&corpo, env!("CARGO_PKG_VERSION")),
        Err(ErroRede::SemConexao(e)) => {
            log(&format!("sem conexao ao verificar atualizacao: {e}"));
            Resultado::SemConexao
        }
        Err(ErroRede::Outro(e)) => {
            log(&format!("falha ao verificar atualizacao: {e}"));
            Resultado::Falhou
        }
    }
}

pub fn interpretar(corpo: &str, local: &str) -> Resultado {
    let release: Release = match serde_json::from_str(corpo) {
        Ok(r) => r,
        Err(e) => {
            log(&format!("resposta ilegivel do GitHub: {e}"));
            return Resultado::Falhou;
        }
    };
    let (Some(remota), Some(atual)) = (versao::interpretar(&release.tag_name), versao::interpretar(local)) else {
        log(&format!("versao ilegivel na consulta: tag={} local={local}", release.tag_name));
        return Resultado::Falhou;
    };
    if !versao::mais_nova(&remota, &atual) {
        return Resultado::Atualizado;
    }

    let asset = release.assets.iter().find(|a| a.name == ASSET && a.browser_download_url.is_some());
    if asset.is_none() {
        log(&format!("release {} sem o asset {ASSET}", release.tag_name));
    }
    Resultado::TemNova(Novidade {
        versao: release.tag_name.trim_start_matches(['v', 'V']).to_owned(),
        url: asset.and_then(|a| a.browser_download_url.clone()),
        // "sha256:<hex>". Campo recente da API: quando nao vier, baixa sem conferir.
        sha256: asset
            .and_then(|a| a.digest.as_deref())
            .and_then(|d| d.get(..7).filter(|p| p.eq_ignore_ascii_case("sha256:")).map(|_| d[7..].to_owned())),
    })
}

/// Baixa o exe novo ao lado do atual e confere o SHA-256 publicado no release. O hash
/// vem da mesma resposta que deu a URL, entao nao protege contra um GitHub comprometido; pega
/// download truncado, proxy que devolve pagina de erro com 200 e cache envenenado no caminho.
pub fn baixar(plataforma: &dyn Plataforma, nova: &Novidade, progresso: &mut dyn FnMut(u32)) -> Option<PathBuf> {
    let url = nova.url.as_deref()?;
    let Some(destino) = plataforma.destino_atualizacao() else {
        log("pasta do app sem permissao de escrita, atualizacao pelo navegador");
        return None;
    };

    let mut ultimo = u32::MAX;
    let mut avisar = |lidos: u64, total: u64| {
        if total == 0 {
            return;
        }
        let pct = (lidos * 100 / total).min(100) as u32;
        if pct != ultimo {
            ultimo = pct;
            progresso(pct);
        }
    };

    if let Err(e) = plataforma.baixar(url, &user_agent(), &destino, &mut avisar) {
        log(&format!("falha ao baixar atualizacao: {e}"));
        let _ = fs::remove_file(&destino);
        return None;
    }

    if let Some(esperado) = nova.sha256.as_deref().filter(|s| !s.is_empty())
        && !confere(&destino, esperado)
    {
        log("sha256 do exe nao bate com o release, download descartado");
        let _ = fs::remove_file(&destino);
        return None;
    }
    log(&format!("versao {} baixada em {}", nova.versao, destino.display()));
    Some(destino)
}

fn confere(caminho: &std::path::Path, esperado: &str) -> bool {
    let Ok(mut arquivo) = fs::File::open(caminho) else { return false };
    let mut hash = Sha256::new();
    let mut buffer = vec![0u8; 64 * 1024];
    loop {
        match arquivo.read(&mut buffer) {
            Ok(0) => break,
            Ok(n) => hash.update(&buffer[..n]),
            Err(_) => return false,
        }
    }
    let hex: String = hash.finalize().iter().map(|b| format!("{b:02x}")).collect();
    hex.eq_ignore_ascii_case(esperado)
}

#[cfg(test)]
mod testes {
    use super::*;

    const RELEASE: &str = r#"{
        "tag_name": "v1.2.0",
        "assets": [
            {"name": "outro.zip", "browser_download_url": "https://x/outro.zip"},
            {"name": "MouseHider.exe", "browser_download_url": "https://x/MouseHider.exe",
             "digest": "sha256:ABCDEF"}
        ]
    }"#;

    #[test]
    fn acha_o_exe_e_o_hash() {
        let r = interpretar(RELEASE, "1.1.0");
        assert_eq!(r, Resultado::TemNova(Novidade {
            versao: "1.2.0".into(),
            url: Some("https://x/MouseHider.exe".into()),
            sha256: Some("ABCDEF".into()),
        }));
    }

    #[test]
    fn mesma_versao_ou_mais_velha() {
        assert_eq!(interpretar(RELEASE, "1.2.0"), Resultado::Atualizado);
        assert_eq!(interpretar(RELEASE, "2.0"), Resultado::Atualizado);
    }

    #[test]
    fn release_sem_asset_ainda_e_novidade() {
        let r = interpretar(r#"{"tag_name": "v9.0.0", "assets": []}"#, "1.0.0");
        assert!(matches!(r, Resultado::TemNova(Novidade { url: None, .. })));
    }

    #[test]
    fn resposta_estranha() {
        assert_eq!(interpretar("<html>", "1.0.0"), Resultado::Falhou);
        assert_eq!(interpretar(r#"{"tag_name": "nightly"}"#, "1.0.0"), Resultado::Falhou);
    }

    #[test]
    fn hash_do_arquivo() {
        let caminho = std::env::temp_dir().join("mouse-hider-teste-sha.bin");
        fs::write(&caminho, b"abc").unwrap();
        assert!(confere(&caminho, "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD"));
        assert!(!confere(&caminho, "00"));
        let _ = fs::remove_file(caminho);
    }
}
