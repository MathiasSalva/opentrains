//! Insignia con los minutos del próximo tren, dibujada sobre el ícono de la bandeja.
//! Los dígitos son un mapa de bits de 3×5 escalado: a 16 px de bandeja no hace falta
//! una fuente real.

/// Mapa de bits 3×5 por fila (bit 2 = columna izquierda).
fn glifo(c: char) -> Option<[u8; 5]> {
    Some(match c {
        '0' => [0b111, 0b101, 0b101, 0b101, 0b111],
        '1' => [0b010, 0b110, 0b010, 0b010, 0b111],
        '2' => [0b111, 0b001, 0b111, 0b100, 0b111],
        '3' => [0b111, 0b001, 0b111, 0b001, 0b111],
        '4' => [0b101, 0b101, 0b111, 0b001, 0b001],
        '5' => [0b111, 0b100, 0b111, 0b001, 0b111],
        '6' => [0b111, 0b100, 0b111, 0b101, 0b111],
        '7' => [0b111, 0b001, 0b010, 0b010, 0b010],
        '8' => [0b111, 0b101, 0b111, 0b101, 0b111],
        '9' => [0b111, 0b101, 0b111, 0b001, 0b111],
        'x' => [0b101, 0b101, 0b010, 0b101, 0b101],
        _ => return None,
    })
}

/// Lado al que se lleva el ícono antes de dibujar: con uno más grande, Windows lo achica
/// para la bandeja y los dígitos se vuelven ilegibles.
pub const LADO_BANDEJA: u32 = 32;

/// Reduce una imagen RGBA cuadrada promediando bloques; si ya es chica, la deja igual.
pub fn reducir(base: &[u8], lado: u32, destino: u32) -> (Vec<u8>, u32) {
    if lado <= destino {
        return (base.to_vec(), lado);
    }
    let mut px = vec![0u8; (destino * destino * 4) as usize];
    for y in 0..destino {
        for x in 0..destino {
            let (x0, x1) = (
                x * lado / destino,
                ((x + 1) * lado / destino).max(x * lado / destino + 1),
            );
            let (y0, y1) = (
                y * lado / destino,
                ((y + 1) * lado / destino).max(y * lado / destino + 1),
            );
            let mut suma = [0u32; 4];
            for sy in y0..y1 {
                for sx in x0..x1 {
                    let i = ((sy * lado + sx) * 4) as usize;
                    for c in 0..4 {
                        suma[c] += base[i + c] as u32;
                    }
                }
            }
            let n = (x1 - x0) * (y1 - y0);
            let i = ((y * destino + x) * 4) as usize;
            for c in 0..4 {
                px[i + c] = (suma[c] / n) as u8;
            }
        }
    }
    (px, destino)
}

/// Dibuja un círculo de `color` en la esquina inferior derecha con `texto` en blanco.
pub fn dibujar(base: &[u8], ancho: u32, alto: u32, texto: &str, color: [u8; 3]) -> Vec<u8> {
    let mut px = base.to_vec();
    let (w, h) = (ancho as i32, alto as i32);
    let lado = w.min(h);
    let radio = lado as f32 * 0.34;
    let (cx, cy) = (w as f32 - radio, h as f32 - radio);
    let mut pintar = |x: i32, y: i32, rgb: [u8; 3]| {
        if (0..w).contains(&x) && (0..h).contains(&y) {
            let i = ((y * w + x) * 4) as usize;
            px[i..i + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
        }
    };

    // Círculo con un borde blanco fino para que se despegue del glifo del tren.
    let borde = (lado as f32 / 32.0).max(1.0);
    for y in 0..h {
        for x in 0..w {
            let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
            if d <= radio - borde {
                pintar(x, y, color);
            } else if d <= radio {
                pintar(x, y, [255, 255, 255]);
            }
        }
    }

    let glifos: Vec<[u8; 5]> = texto.chars().filter_map(glifo).collect();
    if glifos.is_empty() {
        return px;
    }
    let columnas = glifos.len() as i32 * 4 - 1;
    let disponible = radio * 2.0 * 0.72;
    let escala = ((disponible / columnas as f32).min(disponible / 5.0) as i32).max(1);
    let x0 = (cx - (columnas * escala) as f32 / 2.0).round() as i32;
    let y0 = (cy - (5 * escala) as f32 / 2.0).round() as i32;
    for (n, filas) in glifos.iter().enumerate() {
        for (fila, bits) in filas.iter().enumerate() {
            for col in 0..3 {
                if bits & (0b100 >> col) == 0 {
                    continue;
                }
                let x = x0 + (n as i32 * 4 + col) * escala;
                let y = y0 + fila as i32 * escala;
                for dy in 0..escala {
                    for dx in 0..escala {
                        pintar(x + dx, y + dy, [255, 255, 255]);
                    }
                }
            }
        }
    }
    px
}
