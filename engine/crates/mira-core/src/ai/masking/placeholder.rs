//! Repères de remplacement : libellé par catégorie et repères déjà présents dans un texte.

use std::sync::LazyLock;

use mira_vocabulary::SensitiveCategory;
use regex::Regex;

static EXISTING: LazyLock<Option<Regex>> =
    LazyLock::new(|| Regex::new(r"(?i-u)\[[A-Z]+_[0-9]+\]").ok());

/// Libellé du repère d'une catégorie, comme dans `[IBAN_1]`.
pub(super) fn label(category: SensitiveCategory) -> &'static str {
    match category {
        SensitiveCategory::Iban => "IBAN",
        SensitiveCategory::Card => "CARTE",
        SensitiveCategory::Secret => "SECRET",
        SensitiveCategory::Otp => "CODE",
        SensitiveCategory::Email => "EMAIL",
        SensitiveCategory::Phone => "TEL",
    }
}

/// Repères qui ressemblent aux nôtres, quelle que soit leur casse, et figurent déjà dans le texte d'origine (en majuscules).
pub(super) fn existing(text: &str) -> Vec<String> {
    EXISTING.as_ref().map_or_else(Vec::new, |pattern| {
        pattern
            .find_iter(text)
            .map(|m| m.as_str().to_ascii_uppercase())
            .collect()
    })
}
