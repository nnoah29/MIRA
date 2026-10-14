//! Mise à la taille des lignes de l'en-tête : largeur coupée, blocs tronqués avec « … +N ».

/// Coupe le texte à `width` caractères en terminant par une ellipse si besoin.
pub fn clip(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_string();
    }
    let kept: String = text.chars().take(width.saturating_sub(1)).collect();
    if width == 0 {
        kept
    } else {
        format!("{kept}…")
    }
}

/// Garde les premières lignes d'un bloc dans `room` lignes ; la dernière annonce celles qui manquent.
pub fn fit(mut block: Vec<String>, room: usize) -> Vec<String> {
    if block.len() <= room {
        return block;
    }
    if room == 0 {
        return Vec::new();
    }
    let hidden = block.len() - (room - 1);
    block.truncate(room - 1);
    block.push(format!("… +{hidden}"));
    block
}
