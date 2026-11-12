//! Aritmetica de inatividade. Tudo em u32 com subtracao circular: o contador de ticks do
//! Windows da a volta a cada 49,7 dias.

/// Folga apos o atalho antes de aceitar entrada como "o usuario mexeu". Soltar as teclas do
/// proprio atalho conta como entrada para o GetLastInputInfo; sem esta folga o cursor
/// reaparecia no mesmo instante em que sumiu.
pub const GRACA_MANUAL_MS: u32 = 500;

/// Tempo desde a ultima entrada, em ms.
pub fn desde(agora: u32, ultima: u32) -> u32 {
    agora.wrapping_sub(ultima)
}

/// Houve entrada depois da marca do atalho (passada a folga)? Entao o esconder manual acabou.
/// A ultima entrada e `agora - ocioso`; o teto de i32::MAX descarta entrada anterior a marca,
/// que na subtracao circular vira um numero enorme.
pub fn manual_expirou(agora: u32, ocioso: u32, marca: u32) -> bool {
    let depois = agora.wrapping_sub(ocioso).wrapping_sub(marca);
    depois > GRACA_MANUAL_MS && depois < i32::MAX as u32
}

/// O que o laco de 50 ms deve fazer com o cursor.
#[derive(Debug, PartialEq, Eq)]
pub enum Acao {
    Esconder,
    Mostrar,
}

pub fn decidir(pausado: bool, manual: bool, ocioso_ms: u32, limite_s: u32) -> Acao {
    if !pausado && (manual || ocioso_ms >= limite_s.saturating_mul(1000)) { Acao::Esconder } else { Acao::Mostrar }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn idle_simples_e_wraparound() {
        assert_eq!(desde(5000, 1000), 4000);
        assert_eq!(desde(i32::MIN as u32, i32::MAX as u32), 1);
    }

    #[test]
    fn manual_sobrevive_a_soltura_das_teclas() {
        assert!(!manual_expirou(10_600, 500, 10_000));
    }

    #[test]
    fn manual_acaba_no_movimento_seguinte() {
        assert!(manual_expirou(13_000, 100, 10_000));
    }

    #[test]
    fn manual_no_wraparound() {
        assert!(!manual_expirou(i32::MIN as u32, 100, i32::MAX as u32));
    }

    #[test]
    fn decisao() {
        assert_eq!(decidir(false, false, 4999, 5), Acao::Mostrar);
        assert_eq!(decidir(false, false, 5000, 5), Acao::Esconder);
        assert_eq!(decidir(false, true, 0, 5), Acao::Esconder);
        assert_eq!(decidir(true, true, 99_999, 5), Acao::Mostrar);
    }
}
