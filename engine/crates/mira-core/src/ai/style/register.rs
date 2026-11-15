//! Registre de politesse d'un lot de mails : tutoiement, vouvoiement, mélange ou aucune marque.

use mira_vocabulary::StyleRegister;

const TU: [&str; 6] = ["tu", "te", "ton", "ta", "tes", "toi"];
const VOUS: [&str; 3] = ["vous", "votre", "vos"];
const DOMINANT_PERCENT: usize = 70;

/// Nombre de mots de ce mail qui figurent dans la liste.
fn count_in(text: &str, list: &[&str]) -> usize {
    text.split(|c: char| !c.is_alphabetic())
        .filter(|word| list.contains(&word.to_lowercase().as_str()))
        .count()
}

/// Registre d'un seul mail, ou `None` s'il ne porte aucune marque ou en porte autant des deux.
fn mail_register(text: &str) -> Option<StyleRegister> {
    let (tu, vous) = (count_in(text, &TU), count_in(text, &VOUS));
    match tu.cmp(&vous) {
        std::cmp::Ordering::Greater => Some(StyleRegister::Tu),
        std::cmp::Ordering::Less => Some(StyleRegister::Vous),
        std::cmp::Ordering::Equal => None,
    }
}

/// Registre dominant : celui d'au moins 70 % des mails marqués, sinon mélangé ; inconnu sans aucune marque.
pub fn dominant_register(mails: &[String]) -> StyleRegister {
    let marked: Vec<StyleRegister> = mails.iter().filter_map(|m| mail_register(m)).collect();
    let tu = marked.iter().filter(|r| **r == StyleRegister::Tu).count();
    let vous = marked.len() - tu;
    match marked.len() {
        0 => StyleRegister::Unknown,
        n if tu * 100 >= n * DOMINANT_PERCENT => StyleRegister::Tu,
        n if vous * 100 >= n * DOMINANT_PERCENT => StyleRegister::Vous,
        _ => StyleRegister::Mixed,
    }
}
