//! Couleur ARGB d'un libellé écrite `#AARRGGBB`, lue aussi en `#RRGGBB` (opaque).

/// Couleur en texte `#aarrggbb`, huit chiffres hexadécimaux en minuscules
pub fn format_color(color: u32) -> String {
    format!("#{color:08x}")
}

/// Couleur lue en `#RRGGBB` (opaque) ou `#AARRGGBB`, casse libre ; toute autre forme donne `None`
pub fn parse_color(text: &str) -> Option<u32> {
    let digits = text.strip_prefix('#')?;
    if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    match digits.len() {
        6 => u32::from_str_radix(digits, 16)
            .ok()
            .map(|rgb| 0xFF00_0000 | rgb),
        8 => u32::from_str_radix(digits, 16).ok(),
        _ => None,
    }
}
