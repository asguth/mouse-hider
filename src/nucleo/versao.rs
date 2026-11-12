//! Comparacao de versoes "1.2.3" (tag do GitHub com ou sem "v").

/// Partes numericas da versao, ou None se o texto nao for uma versao.
pub fn interpretar(texto: &str) -> Option<Vec<u64>> {
    let limpa = texto.trim().trim_start_matches(['v', 'V']);
    let partes: Option<Vec<u64>> = limpa.split('.').map(|p| p.parse().ok()).collect();
    partes.filter(|p| (2..=4).contains(&p.len()))
}

/// A remota e mais nova que a local? Partes faltando contam como zero (1.1 == 1.1.0).
pub fn mais_nova(remota: &[u64], local: &[u64]) -> bool {
    let n = remota.len().max(local.len());
    let parte = |v: &[u64], i: usize| v.get(i).copied().unwrap_or(0);
    (0..n).map(|i| parte(remota, i).cmp(&parte(local, i))).find(|o| o.is_ne()).is_some_and(|o| o.is_gt())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn compara() {
        let v = |t| interpretar(t).unwrap();
        assert!(mais_nova(&v("v1.1.0"), &v("1.0.2")));
        assert!(mais_nova(&v("1.0.10"), &v("1.0.9")));
        assert!(!mais_nova(&v("1.1"), &v("1.1.0")));
        assert!(!mais_nova(&v("1.0.2"), &v("1.0.2")));
        assert!(interpretar("latest").is_none());
        assert!(interpretar("1").is_none());
    }
}
