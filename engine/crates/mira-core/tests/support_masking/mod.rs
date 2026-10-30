//! Outils des tests de masquage : valeurs trouvées, masqueur prêt et restitution d'un flux découpé.
#![allow(dead_code)]

use mira_core::ai::masking::{Masker, Restorer, detect};
use mira_vocabulary::{MaskingMode, MaskingSettings, SensitiveCategory};

/// Toutes les catégories connues
pub const ALL: [SensitiveCategory; 6] = [
    SensitiveCategory::Iban,
    SensitiveCategory::Card,
    SensitiveCategory::Secret,
    SensitiveCategory::Otp,
    SensitiveCategory::Email,
    SensitiveCategory::Phone,
];

/// Textes trouvés dans ce texte pour ces catégories, dans l'ordre du texte
pub fn values<'a>(text: &'a str, categories: &[SensitiveCategory]) -> Vec<&'a str> {
    detect(text, categories)
        .iter()
        .map(|found| &text[found.start..found.end])
        .collect()
}

/// Réglage actif sur ces catégories
pub fn mask_on(categories: &[SensitiveCategory]) -> MaskingSettings {
    MaskingSettings {
        mode: MaskingMode::Mask,
        categories: categories.to_vec(),
    }
}

/// Masqueur actif sur toutes les catégories
pub fn masker_all() -> Masker {
    Masker::new(&mask_on(&ALL))
}

/// Restituteur d'une table dont les repères sont obtenus en masquant ce texte
pub fn restorer_for(text: &str) -> (String, Restorer) {
    let mut masker = masker_all();
    let masked = masker.mask(text);
    (masked, masker.into_restorer())
}

/// Passe le texte par `push` en le coupant en morceaux de ces tailles (en caractères), puis `finish`
pub fn feed(restorer: &mut Restorer, text: &str, cut: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out: String = chars
        .chunks(cut.max(1))
        .map(|chunk| restorer.push(&chunk.iter().collect::<String>()))
        .collect();
    out.push_str(&restorer.finish());
    out
}
