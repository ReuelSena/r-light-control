//! O texto do RGB Matrix, como o `RGBText` do QLC+, com uma fonte de pontos 5 × 7 própria no lugar das
//! fontes do Qt (o resultado não é igual pixel a pixel ao do QLC+). Letras minúsculas saem como as
//! maiúsculas, e as acentuadas, sem o acento; o que a fonte não tem sai em branco.

use rlc_core::ScrollStyle;

use crate::matrix::Map;

/// Largura de um caractere (5 pontos e 1 de espaço) e altura da fonte.
const ADVANCE: i32 = 6;
const HEIGHT: i32 = 7;
const WIDTH: i32 = 5;

/// Quantos passos tem o texto: uma letra, a largura do texto em pontos (rolando para o lado) ou a altura
/// das letras uma sobre a outra (rolando para cima).
pub fn steps(text: &str, style: ScrollStyle) -> usize {
    let letters = text.chars().count();
    match style {
        ScrollStyle::Static => letters,
        ScrollStyle::Horizontal => letters * ADVANCE as usize,
        ScrollStyle::Vertical => letters * HEIGHT as usize,
    }
}

/// O mapa do passo `step` numa grade `width` × `height`, como o `renderStaticLetters` e o
/// `renderScrollingText` do QLC+: a grade é uma janela sobre o texto inteiro desenhado, que anda um
/// ponto por passo. Fora do texto, a célula fica apagada.
pub fn map(
    text: &str,
    style: ScrollStyle,
    offset: (i32, i32),
    size: (u16, u16),
    rgb: u32,
    step: usize,
) -> Map {
    let (width, height) = (i32::from(size.0), i32::from(size.1));
    let (dx, dy) = offset;
    let step = step as i32;
    let letters: Vec<char> = text.chars().collect();
    // O ponto (col, row) do texto inteiro, no sistema de cada estilo
    let lit = |x: i32, y: i32| -> bool {
        match style {
            // Uma letra por passo, no centro da grade
            ScrollStyle::Static => {
                let left = dx + (width - WIDTH).div_euclid(2);
                let top = dy + (height - HEIGHT).div_euclid(2);
                letters
                    .get(step as usize)
                    .is_some_and(|&c| pixel(c, x - left, y - top))
            }
            // O texto inteiro à esquerda, no meio da altura, e a grade anda sobre ele
            ScrollStyle::Horizontal => {
                let (vx, vy) = (step + x, y);
                let top = dy + (height - HEIGHT).div_euclid(2);
                vx < letters.len() as i32 * ADVANCE && {
                    let (col, i) = ((vx - dx).rem_euclid(ADVANCE), (vx - dx).div_euclid(ADVANCE));
                    usize::try_from(i)
                        .ok()
                        .and_then(|i| letters.get(i))
                        .is_some_and(|&c| pixel(c, col, vy - top))
                }
            }
            // As letras uma sobre a outra, cada uma na sua altura, e a grade anda para baixo
            ScrollStyle::Vertical => {
                let (vx, vy) = (x, step + y);
                vy < letters.len() as i32 * HEIGHT && {
                    let (row, i) = ((vy - dy).rem_euclid(HEIGHT), (vy - dy).div_euclid(HEIGHT));
                    usize::try_from(i)
                        .ok()
                        .and_then(|i| letters.get(i))
                        .is_some_and(|&c| pixel(c, vx - dx, row))
                }
            }
        }
    };
    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| if lit(x, y) { rgb } else { 0 })
                .collect()
        })
        .collect()
}

/// O ponto (`col`, `row`) da letra: aceso ou apagado; fora dos 5 × 7, apagado.
fn pixel(c: char, col: i32, row: i32) -> bool {
    if !(0..WIDTH).contains(&col) || !(0..HEIGHT).contains(&row) {
        return false;
    }
    glyph(c)[row as usize] >> (WIDTH - 1 - col) & 1 == 1
}

/// As 7 linhas da letra, de cima para baixo, com um bit por ponto (o da esquerda é o 5º).
fn glyph(c: char) -> [u8; 7] {
    // Maiúscula, sem acento
    let c = c.to_uppercase().next().unwrap_or(' ');
    let c = match c {
        'Á' | 'À' | 'Â' | 'Ã' | 'Ä' => 'A',
        'É' | 'È' | 'Ê' | 'Ë' => 'E',
        'Í' | 'Ì' | 'Î' | 'Ï' => 'I',
        'Ó' | 'Ò' | 'Ô' | 'Õ' | 'Ö' => 'O',
        'Ú' | 'Ù' | 'Û' | 'Ü' => 'U',
        'Ç' => 'C',
        c => c,
    };
    match c {
        '0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
        '1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
        '2' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1F],
        '3' => [0x1F, 0x02, 0x04, 0x02, 0x01, 0x11, 0x0E],
        '4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        '5' => [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x11, 0x0E],
        '6' => [0x06, 0x08, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        '7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        '9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x02, 0x0C],
        'A' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
        'C' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
        'D' => [0x1C, 0x12, 0x11, 0x11, 0x11, 0x12, 0x1C],
        'E' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
        'F' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10],
        'G' => [0x0E, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0F],
        'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'I' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
        'J' => [0x07, 0x02, 0x02, 0x02, 0x02, 0x12, 0x0C],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        'M' => [0x11, 0x1B, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x11, 0x19, 0x15, 0x13, 0x11, 0x11],
        'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
        'Q' => [0x0E, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0D],
        'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        'S' => [0x0F, 0x10, 0x10, 0x0E, 0x01, 0x01, 0x1E],
        'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x15, 0x0A],
        'X' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x11, 0x0A, 0x04, 0x04, 0x04],
        'Z' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C],
        ',' => [0x00, 0x00, 0x00, 0x00, 0x0C, 0x04, 0x08],
        '!' => [0x04, 0x04, 0x04, 0x04, 0x04, 0x00, 0x04],
        '?' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x00, 0x04],
        '-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
        '+' => [0x00, 0x04, 0x04, 0x1F, 0x04, 0x04, 0x00],
        '=' => [0x00, 0x00, 0x1F, 0x00, 0x1F, 0x00, 0x00],
        ':' => [0x00, 0x0C, 0x0C, 0x00, 0x0C, 0x0C, 0x00],
        '/' => [0x01, 0x01, 0x02, 0x04, 0x08, 0x10, 0x10],
        '(' => [0x02, 0x04, 0x08, 0x08, 0x08, 0x04, 0x02],
        ')' => [0x08, 0x04, 0x02, 0x02, 0x02, 0x04, 0x08],
        '\'' => [0x04, 0x04, 0x08, 0x00, 0x00, 0x00, 0x00],
        '*' => [0x00, 0x04, 0x15, 0x0E, 0x15, 0x04, 0x00],
        _ => [0; 7],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ScrollStyle::*;

    /// O mapa como linhas de texto: `#` aceso, `.` apagado.
    fn art(map: &Map) -> Vec<String> {
        map.iter()
            .map(|row| {
                row.iter()
                    .map(|&c| if c == 0 { '.' } else { '#' })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn the_steps_follow_the_style_like_the_qlc() {
        assert_eq!(steps("OLÁ", Static), 3);
        assert_eq!(steps("OLÁ", Horizontal), 18);
        assert_eq!(steps("OLÁ", Vertical), 21);
        assert_eq!(steps("", Horizontal), 0);
    }

    #[test]
    fn a_static_letter_is_centered_in_the_grid() {
        // "A" no meio de uma grade 7 × 9: o 5 × 7 da letra, com uma coluna e uma linha de cada lado
        let map = map("A", Static, (0, 0), (7, 9), 1, 0);
        assert_eq!(
            art(&map),
            [
                ".......", "..###..", ".#...#.", ".#...#.", ".#####.", ".#...#.", ".#...#.",
                ".#...#.", "......."
            ]
        );
        // Sem a letra do passo, nada acende; o deslocamento anda a letra
        assert!(
            art(&super::map("A", Static, (0, 0), (7, 9), 1, 1))
                .iter()
                .all(|r| !r.contains('#'))
        );
        let moved = art(&super::map("A", Static, (1, -1), (7, 9), 1, 0));
        assert_eq!(moved[0], "...###.");
    }

    #[test]
    fn horizontal_text_slides_one_dot_per_step() {
        // "IA" numa grade 12 × 7: no passo 0, o I e o A lado a lado; no passo 6, o A à esquerda
        let first = art(&map("IA", Horizontal, (0, 0), (12, 7), 1, 0));
        assert_eq!(first[0], ".###...###..");
        assert_eq!(first[3], "..#...#####.");
        let sixth = art(&map("IA", Horizontal, (0, 0), (12, 7), 1, 6));
        assert_eq!(sixth[3], "#####.......");
        // No fim do texto só sobra a folga do último caractere: apagado
        assert!(
            art(&map("IA", Horizontal, (0, 0), (12, 7), 1, 11))
                .iter()
                .all(|r| !r.contains('#'))
        );
    }

    #[test]
    fn vertical_text_stacks_one_letter_over_the_other() {
        // "IA" numa grade 5 × 7: no passo 0, o I; no passo 7, o A
        let first = art(&map("IA", Vertical, (0, 0), (5, 7), 1, 0));
        assert_eq!(first[0], ".###.");
        assert_eq!(first[3], "..#..");
        let second = art(&map("IA", Vertical, (0, 0), (5, 7), 1, 7));
        assert_eq!(second[0], ".###.");
        assert_eq!(second[3], "#####");
    }

    #[test]
    fn accents_fold_and_lowercase_matches_uppercase() {
        assert_eq!(glyph('ç'), glyph('C'));
        assert_eq!(glyph('Ã'), glyph('A'));
        assert_eq!(glyph('ü'), glyph('U'));
        assert_eq!(glyph('é'), glyph('E'));
        assert_eq!(glyph('@'), [0; 7]);
    }
}
