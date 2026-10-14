//! Passage d'un indice de caractère à un décalage en octets, jamais l'inverse sans contrôle.

/// Décalage en octets du caractère `col` de la ligne, ou la longueur de la ligne s'il n'y en a pas autant.
pub fn byte_offset(line: &str, col: usize) -> usize {
    line.char_indices()
        .nth(col)
        .map_or(line.len(), |(offset, _)| offset)
}

/// Nombre de caractères de la ligne.
pub fn char_len(line: &str) -> usize {
    line.chars().count()
}
