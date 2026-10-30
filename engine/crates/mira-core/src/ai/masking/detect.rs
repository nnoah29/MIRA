//! Détection des données sensibles : chaque catégorie active cherche, la plus prioritaire gagne en cas de chevauchement.

use mira_vocabulary::SensitiveCategory;

use super::{card, contact, iban, otp, secret};

const PRIORITY: [SensitiveCategory; 6] = [
    SensitiveCategory::Iban,
    SensitiveCategory::Card,
    SensitiveCategory::Secret,
    SensitiveCategory::Otp,
    SensitiveCategory::Email,
    SensitiveCategory::Phone,
];

/// Une valeur sensible trouvée : octets `start..end` du texte et sa catégorie.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Found {
    pub start: usize,
    pub end: usize,
    pub category: SensitiveCategory,
}

/// Trouve les valeurs sensibles des catégories données, sans chevauchement, dans l'ordre du texte.
pub fn detect(text: &str, categories: &[SensitiveCategory]) -> Vec<Found> {
    let mut accepted: Vec<Found> = Vec::new();
    for category in PRIORITY.iter().filter(|c| categories.contains(c)) {
        let mut spans = spans_of(*category, text);
        spans.sort_unstable();
        for (start, end) in spans.into_iter().filter(|(start, end)| start < end) {
            if !accepted.iter().any(|f| start < f.end && f.start < end) {
                accepted.push(Found {
                    start,
                    end,
                    category: *category,
                });
            }
        }
    }
    accepted.sort_by_key(|found| found.start);
    accepted
}

/// Emplacements candidats d'une catégorie, par le détecteur qui la concerne.
fn spans_of(category: SensitiveCategory, text: &str) -> Vec<(usize, usize)> {
    match category {
        SensitiveCategory::Iban => iban::find(text),
        SensitiveCategory::Card => card::find(text),
        SensitiveCategory::Secret => secret::find(text),
        SensitiveCategory::Otp => otp::find(text),
        SensitiveCategory::Email => contact::find_email(text),
        SensitiveCategory::Phone => contact::find_phone(text),
    }
}
