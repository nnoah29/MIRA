//! Noms de boîtes en UTF-7 modifié (RFC 3501 §5.1.3) : texte lisible côté moteur, UTF-7 sur le fil.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD_NO_PAD;

/// Encode un nom lisible : ASCII imprimable tel quel, `&` doublé, le reste en blocs UTF-16 base64.
pub(crate) fn encode_mailbox_name(name: &str) -> String {
    let mut encoded = String::new();
    let mut pending: Vec<u16> = Vec::new();
    for character in name.chars() {
        if !matches!(character, ' '..='~') {
            let mut units = [0u16; 2];
            pending.extend_from_slice(character.encode_utf16(&mut units));
            continue;
        }
        flush_shifted(&mut encoded, &mut pending);
        encoded.push(character);
        if character == '&' {
            encoded.push('-');
        }
    }
    flush_shifted(&mut encoded, &mut pending);
    encoded
}

/// Écrit les unités UTF-16 en attente comme un bloc `&…-` et vide la file
fn flush_shifted(encoded: &mut String, pending: &mut Vec<u16>) {
    if pending.is_empty() {
        return;
    }
    let bytes: Vec<u8> = pending.iter().flat_map(|unit| unit.to_be_bytes()).collect();
    encoded.push('&');
    encoded.push_str(&STANDARD_NO_PAD.encode(bytes).replace('/', ","));
    encoded.push('-');
    pending.clear();
}

/// Décode un nom reçu du serveur ; un nom mal formé est gardé tel quel
pub(crate) fn decode_mailbox_name(name: &str) -> String {
    decode_runs(name).unwrap_or_else(|| name.to_string())
}

/// Décode tous les blocs `&…-` du nom, ou `None` si l'un d'eux est invalide
fn decode_runs(name: &str) -> Option<String> {
    let mut decoded = String::new();
    let mut rest = name;
    while let Some(start) = rest.find('&') {
        decoded.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let end = after.find('-')?;
        decoded.push_str(&decode_run(&after[..end])?);
        rest = &after[end + 1..];
    }
    decoded.push_str(rest);
    Some(decoded)
}

/// Décode le contenu d'un bloc : vide pour `&`, sinon UTF-16 big-endian en base64 modifié
fn decode_run(run: &str) -> Option<String> {
    if run.is_empty() {
        return Some("&".to_string());
    }
    let bytes = STANDARD_NO_PAD.decode(run.replace(',', "/")).ok()?;
    if bytes.len() % 2 != 0 {
        return None;
    }
    let units: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_be_bytes(*pair))
        .collect();
    String::from_utf16(&units).ok()
}
