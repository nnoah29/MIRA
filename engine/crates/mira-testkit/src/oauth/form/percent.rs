//! Encodage et décodage en pourcentage (RFC 3986, avec `+` pour l'espace en formulaire).

use std::str::Bytes;

/// Décode `%XX` et `+` ; une séquence invalide devient `?`
pub fn decode(text: &str) -> String {
    let mut bytes = text.bytes();
    let mut out = Vec::with_capacity(text.len());
    while let Some(byte) = bytes.next() {
        match byte {
            b'+' => out.push(b' '),
            b'%' => out.push(decode_escape(&mut bytes)),
            other => out.push(other),
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Lit les deux chiffres hexadécimaux qui suivent un `%`
fn decode_escape(bytes: &mut Bytes<'_>) -> u8 {
    let high = bytes.next().and_then(hex_value);
    let low = bytes.next().and_then(hex_value);
    match (high, low) {
        (Some(high), Some(low)) => high * 16 + low,
        _ => b'?',
    }
}

/// Valeur d'un chiffre hexadécimal ASCII
fn hex_value(byte: u8) -> Option<u8> {
    char::from(byte)
        .to_digit(16)
        .and_then(|digit| u8::try_from(digit).ok())
}

/// Encode tout octet hors lettres, chiffres et `-._~` en `%XX`
pub fn encode(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for byte in text.bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}
