//! O que a interface precisa do sistema. No Windows e o app de verdade; fora dele, uma previa
//! falsa para desenvolver e tirar screenshots da tela.

use std::path::{Path, PathBuf};

/// Falha de rede separada de resposta estranha: na tela uma e "Falha na conexao", a outra
/// e "nao deu para verificar".
#[derive(Debug)]
pub enum ErroRede {
    SemConexao(String),
    Outro(String),
}

pub trait Plataforma: Send + Sync {
    /// Milissegundos desde a ultima entrada de mouse ou teclado.
    fn ocioso_ms(&self) -> u32;
    fn cursor_escondido(&self) -> bool;

    /// Registra o atalho global. false = combinacao tomada por outro app.
    fn aplicar_atalho(&self, atalho: &str) -> bool;
    /// Solta o atalho enquanto a tela captura um novo (senao a combinacao atual nem chega na janela).
    fn soltar_atalho(&self);

    fn autostart(&self) -> bool;
    fn definir_autostart(&self, ligado: bool);

    /// Pausa ou retoma; o chamador ja gravou no config. Atualiza bandeja e cursor.
    fn pausa_alterada(&self);
    /// Idioma trocado: refaz os textos da bandeja.
    fn idioma_alterado(&self);
    /// Idioma da interface do sistema, em duas letras.
    fn idioma_do_sistema(&self) -> &'static str;

    fn abrir_url(&self, url: &str);

    /// Corpo JSON do release mais recente.
    fn buscar_release(&self, user_agent: &str) -> Result<String, ErroRede>;
    /// Onde gravar o exe novo, ou None se a pasta do app nao aceitar escrita.
    fn destino_atualizacao(&self) -> Option<PathBuf>;
    /// Baixa a URL para o destino, avisando (lidos, total). Total 0 = desconhecido.
    fn baixar(&self, url: &str, user_agent: &str, destino: &Path, progresso: &mut dyn FnMut(u64, u64)) -> Result<(), String>;
    /// Poe o exe baixado no lugar do atual e abre a versao nova. false = nada foi trocado.
    fn instalar(&self, novo: &Path) -> bool;

    /// Chamado logo antes de a janela aparecer.
    fn antes_de_mostrar(&self) {}
    /// Encerra o app, sempre com o cursor restaurado.
    fn sair(&self);
}
