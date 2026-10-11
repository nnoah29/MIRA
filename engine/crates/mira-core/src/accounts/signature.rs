//! Normalisation des champs saisis d'une identité.

/// Une signature vide ou blanche signifie « aucune signature »
pub fn normalize_signature(signature: Option<String>) -> Option<String> {
    signature.filter(|text| !text.trim().is_empty())
}
