//! Affichage d'un champ secret : des pastilles, jamais les caractères saisis.

const BULLET: char = '•';

/// Une pastille par caractère saisi, au plus `width` ; les plus anciennes ne sont pas montrées si la place manque.
pub fn mask(char_count: usize, width: usize) -> String {
    std::iter::repeat_n(BULLET, char_count.min(width)).collect()
}
