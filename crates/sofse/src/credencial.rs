//! Credencial de cliente diaria (ver `docs/reverse-engineering/trenes-argentinos-api.md` §2).

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chrono::{DateTime, FixedOffset, NaiveDate, Utc};

const SUFIJO: &str = "sofse";

/// Usuario y clave (ya codificada para el body) del día dado.
pub fn para_fecha(fecha: NaiveDate) -> (String, String) {
    let usuario = STANDARD.encode(format!("{}{SUFIJO}", fecha.format("%Y%m%d")));
    let clave = codificar_clave(&usuario);
    (usuario, clave)
}

/// La API calcula la credencial con la fecha de Argentina (UTC-3, sin horario de verano).
pub fn fecha_argentina(momento: DateTime<Utc>) -> NaiveDate {
    let argentina = FixedOffset::west_opt(3 * 3600).expect("offset válido");
    momento.with_timezone(&argentina).date_naive()
}

fn codificar_clave(usuario: &str) -> String {
    let paso = STANDARD.encode(usuario);
    let paso = reemplazar(
        &paso,
        &[
            ("a", "#t"),
            ("e", "#x"),
            ("i", "#f"),
            ("o", "#l"),
            ("u", "#7"),
            ("=", "#g"),
        ],
    );
    let paso = invertir(&paso);
    let paso = STANDARD.encode(paso);
    let paso = reemplazar(
        &paso,
        &[
            ("a", "#j"),
            ("e", "#p"),
            ("i", "#w"),
            ("o", "#8"),
            ("u", "#0"),
            ("=", "#v"),
        ],
    );
    encode_uri_component(&invertir(&paso))
}

fn reemplazar(texto: &str, pares: &[(&str, &str)]) -> String {
    pares
        .iter()
        .fold(texto.to_owned(), |acc, (de, a)| acc.replace(de, a))
}

fn invertir(texto: &str) -> String {
    texto.chars().rev().collect()
}

/// Equivalente a `encodeURIComponent` de JavaScript para texto ASCII.
fn encode_uri_component(texto: &str) -> String {
    texto
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'_'
            | b'.'
            | b'!'
            | b'~'
            | b'*'
            | b'\''
            | b'('
            | b')' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}
