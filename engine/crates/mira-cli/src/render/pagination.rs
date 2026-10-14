//! Pied de page des listes paginées : comment demander la page suivante.

use mira_assembly::mira_vocabulary::Cursor;

/// Ajoute au tableau la commande pour la page suivante, s'il y en a une.
pub fn with_next_page(table: String, next: Option<&Cursor>) -> String {
    match next {
        Some(cursor) => format!("{table}\nsuite : --cursor {}", cursor.0),
        None => table,
    }
}
