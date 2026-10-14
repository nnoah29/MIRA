//! Titre de la liste : « Messages », ou la requête pendant une recherche.

use crate::state::AppState;
use crate::view::safe_text::display_safe;

/// Titre du cadre de la liste ; la requête saisie est neutralisée avant affichage.
pub fn list_title(state: &AppState) -> String {
    match state.search.query() {
        Some(query) => match state.search.first_page_only() {
            Some(count) => format!(
                "Recherche : {} ({count} premiers résultats)",
                display_safe(query)
            ),
            None => format!("Recherche : {}", display_safe(query)),
        },
        None => "Messages".to_string(),
    }
}
