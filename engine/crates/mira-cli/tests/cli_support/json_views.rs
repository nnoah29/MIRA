//! Extraction de valeurs dans les réponses JSON du contrat et composition de lignes de commande.

use serde_json::Value;

/// Identifiants des éléments d'une page (`data.items`), dans l'ordre rendu.
pub fn page_ids(data: &Value) -> Vec<String> {
    let items = data["data"]["items"].as_array().unwrap();
    items
        .iter()
        .map(|item| item["id"].as_str().unwrap().to_string())
        .collect()
}

/// Ligne de commande formée de `base` suivie de `extra`.
pub fn with_args<'a>(base: &[&'a str], extra: &[&'a str]) -> Vec<&'a str> {
    let mut line = base.to_vec();
    line.extend_from_slice(extra);
    line
}

/// Mots d'une ligne de commande écrite en une seule chaîne (sans espace dans les valeurs).
pub fn words(line: &str) -> Vec<&str> {
    line.split_whitespace().collect()
}
