//! O logo desenhado por codigo: um cursor branco que se dissolve sobre um quadrado de cantos
//! suaves, em cor chapada. Nada de asset binario para manter: o .ico sai de `--makeicon`, a
//! bandeja e a janela renderizam direto em memoria.

use tiny_skia::{
    Color, FillRule, GradientStop, LinearGradient, Paint, PathBuilder, Pixmap, Point, Rect, SpreadMode, Transform,
};

/// Verde-petroleo. Unica cor de marca do app.
pub const MARCA: [u8; 3] = [0x0E, 0x7C, 0x70];
const MARCA_PAUSADO: [u8; 3] = [0x8A, 0x8A, 0x85];

/// Contorno do cursor num espaco 12.2 x 20.
const SETA: [(f32, f32); 7] = [(0.0, 0.0), (0.0, 17.5), (4.3, 13.6), (7.3, 20.0), (10.4, 18.5), (7.5, 12.3), (12.2, 11.7)];

/// Logo em RGBA (sem pre-multiplicar), lado x lado.
pub fn rgba(lado: u32, pausado: bool) -> Vec<u8> {
    let pixmap = desenhar(lado, pausado);
    pixmap.pixels().iter().flat_map(|p| {
        let c = p.demultiply();
        [c.red(), c.green(), c.blue(), c.alpha()]
    }).collect()
}

pub fn png(lado: u32) -> Vec<u8> {
    desenhar(lado, false).encode_png().unwrap_or_default()
}

fn desenhar(lado: u32, pausado: bool) -> Pixmap {
    let mut pixmap = Pixmap::new(lado, lado).expect("tamanho de icone valido");
    let s = lado as f32;
    let [r, g, b] = if pausado { MARCA_PAUSADO } else { MARCA };

    // Em 16 px a margem some; acima disso, 1/16 de respiro deixa o icone do tamanho dos vizinhos.
    let margem = if lado <= 16 { 0.0 } else { s / 16.0 };
    let fundo = Rect::from_xywh(margem, margem, s - 2.0 * margem, s - 2.0 * margem).unwrap();
    let mut tinta = Paint { anti_alias: true, ..Paint::default() };
    tinta.set_color_rgba8(r, g, b, 255);
    if let Some(caminho) = arredondado(fundo, fundo.width() * 0.24) {
        pixmap.fill_path(&caminho, &tinta, FillRule::Winding, Transform::identity(), None);
    }

    // Cursor: topo opaco, base translucida. Le como "sumindo" sem depender de detalhe fino.
    let altura = fundo.height() * 0.58;
    let escala = altura / 20.0;
    let largura = 12.2 * escala;
    let x0 = margem + (fundo.width() - largura) / 2.0 + fundo.width() * 0.02;
    let y0 = margem + (fundo.height() - altura) / 2.0;

    let mut seta = PathBuilder::new();
    for (i, (x, y)) in SETA.iter().enumerate() {
        let p = (x0 + x * escala, y0 + y * escala);
        if i == 0 { seta.move_to(p.0, p.1) } else { seta.line_to(p.0, p.1) }
    }
    seta.close();

    let base = if lado <= 24 { 0.55 } else { 0.22 }; // em tamanho pequeno o degrade forte some
    let mut branco = Paint { anti_alias: true, ..Paint::default() };
    branco.shader = LinearGradient::new(
        Point::from_xy(0.0, y0),
        Point::from_xy(0.0, y0 + altura),
        vec![
            GradientStop::new(0.0, Color::WHITE),
            GradientStop::new(0.45, Color::WHITE),
            GradientStop::new(1.0, Color::from_rgba(1.0, 1.0, 1.0, base).unwrap()),
        ],
        SpreadMode::Pad,
        Transform::identity(),
    )
    .expect("degrade valido");
    if let Some(caminho) = seta.finish() {
        pixmap.fill_path(&caminho, &branco, FillRule::Winding, Transform::identity(), None);
    }
    pixmap
}

fn arredondado(r: Rect, raio: f32) -> Option<tiny_skia::Path> {
    // Curva cubica com k = 0.5523 da um quarto de circulo.
    let k = raio * 0.5523;
    let (x0, y0, x1, y1) = (r.left(), r.top(), r.right(), r.bottom());
    let mut p = PathBuilder::new();
    p.move_to(x0 + raio, y0);
    p.line_to(x1 - raio, y0);
    p.cubic_to(x1 - raio + k, y0, x1, y0 + raio - k, x1, y0 + raio);
    p.line_to(x1, y1 - raio);
    p.cubic_to(x1, y1 - raio + k, x1 - raio + k, y1, x1 - raio, y1);
    p.line_to(x0 + raio, y1);
    p.cubic_to(x0 + raio - k, y1, x0, y1 - raio + k, x0, y1 - raio);
    p.line_to(x0, y0 + raio);
    p.cubic_to(x0, y0 + raio - k, x0 + raio - k, y0, x0 + raio, y0);
    p.close();
    p.finish()
}

/// .ico multi-resolucao: DIB classico ate 48 px e PNG so no 256, a convencao que o Explorer,
/// o Inno Setup e o rc.exe aceitam sem engasgar.
pub fn ico() -> Vec<u8> {
    let lados = [16u32, 20, 24, 32, 40, 48, 64, 256];
    let imagens: Vec<Vec<u8>> = lados.iter().map(|&l| if l >= 256 { png(l) } else { dib(l) }).collect();

    let mut saida = Vec::new();
    saida.extend_from_slice(&0u16.to_le_bytes()); // reservado
    saida.extend_from_slice(&1u16.to_le_bytes()); // tipo: icone
    saida.extend_from_slice(&(lados.len() as u16).to_le_bytes());

    let mut deslocamento = 6 + 16 * lados.len() as u32;
    for (lado, img) in lados.iter().zip(&imagens) {
        let byte = if *lado >= 256 { 0 } else { *lado as u8 }; // 0 == 256
        saida.extend_from_slice(&[byte, byte, 0, 0]);
        saida.extend_from_slice(&1u16.to_le_bytes()); // planos
        saida.extend_from_slice(&32u16.to_le_bytes()); // bits por pixel
        saida.extend_from_slice(&(img.len() as u32).to_le_bytes());
        saida.extend_from_slice(&deslocamento.to_le_bytes());
        deslocamento += img.len() as u32;
    }
    for img in &imagens {
        saida.extend_from_slice(img);
    }
    saida
}

/// BITMAPINFOHEADER + BGRA de baixo para cima + mascara AND zerada (o alfa manda).
fn dib(lado: u32) -> Vec<u8> {
    let rgba = rgba(lado, false);
    let linha = lado as usize * 4;
    let mascara = lado.div_ceil(32) as usize * 4 * lado as usize;
    let mut saida = Vec::with_capacity(40 + rgba.len() + mascara);
    for v in [40u32, lado, lado * 2] {
        saida.extend_from_slice(&v.to_le_bytes());
    }
    saida.extend_from_slice(&1u16.to_le_bytes());
    saida.extend_from_slice(&32u16.to_le_bytes());
    for v in [0u32, (rgba.len() + mascara) as u32, 0, 0, 0, 0] {
        saida.extend_from_slice(&v.to_le_bytes());
    }
    for y in (0..lado as usize).rev() {
        for px in rgba[y * linha..(y + 1) * linha].chunks_exact(4) {
            saida.extend_from_slice(&[px[2], px[1], px[0], px[3]]);
        }
    }
    saida.resize(saida.len() + mascara, 0);
    saida
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn ico_tem_cabecalho_coerente() {
        let ico = ico();
        assert_eq!(&ico[..6], &[0, 0, 1, 0, 8, 0]);
        let ultimo = &ico[6 + 16 * 7..6 + 16 * 8];
        let tamanho = u32::from_le_bytes(ultimo[8..12].try_into().unwrap()) as usize;
        let inicio = u32::from_le_bytes(ultimo[12..16].try_into().unwrap()) as usize;
        assert_eq!(inicio + tamanho, ico.len());
        assert_eq!(&ico[inicio + 1..inicio + 4], b"PNG");
    }

    #[test]
    fn centro_e_branco_e_canto_e_transparente() {
        let px = rgba(64, false);
        assert_eq!(px[3], 0, "canto transparente");
        let meio = (24 * 64 + 30) * 4;
        assert_eq!(&px[meio..meio + 4], &[255, 255, 255, 255]);
    }
}
