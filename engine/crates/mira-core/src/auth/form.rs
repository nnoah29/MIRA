//! Encodage et décodage `application/x-www-form-urlencoded` (RFC 3986 pour l'encodage).

/// Encode une valeur : seuls les caractères non réservés restent tels quels
pub fn percent_encode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            out.push(char::from(byte));
        } else {
            out.push_str(&format!("%{byte:02X}"));
        }
    }
    out
}

/// Décode `%XX` et `+` ; une séquence invalide est conservée telle quelle
pub fn percent_decode(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        let hex = bytes.get(index + 1..index + 3).and_then(hex_pair);
        match (byte, hex) {
            (b'%', Some(decoded)) => {
                out.push(decoded);
                index += 3;
            }
            (b'+', _) => {
                out.push(b' ');
                index += 1;
            }
            _ => {
                out.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Valeur d'un octet écrit en deux chiffres hexadécimaux
fn hex_pair(pair: &[u8]) -> Option<u8> {
    if !pair.iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    let text = std::str::from_utf8(pair).ok()?;
    u8::from_str_radix(text, 16).ok()
}

/// Assemble des paires nom et valeur en chaîne de requête encodée
pub fn build_query(pairs: &[(&str, &str)]) -> String {
    let encoded: Vec<String> = pairs
        .iter()
        .map(|(name, value)| format!("{}={}", percent_encode(name), percent_encode(value)))
        .collect();
    encoded.join("&")
}

/// Découpe une chaîne de requête en paires décodées
pub fn parse_query(query: &str) -> Vec<(String, String)> {
    query
        .split('&')
        .filter(|pair| !pair.is_empty())
        .map(|pair| {
            let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
            (percent_decode(name), percent_decode(value))
        })
        .collect()
}
