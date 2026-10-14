//! Rendu humain du résultat d'une recherche sur le serveur.

use mira_assembly::fetching::{ServerSearchHit, ServerSearchResult};

use super::table::render_table;

/// Cellules d'une ligne de résultat.
fn hit_row(hit: &ServerSearchHit) -> Vec<String> {
    vec![
        hit.mailbox.to_string(),
        hit.uid.to_string(),
        hit.message.as_ref().map_or_else(
            || "(pas encore synchronisé)".to_string(),
            ToString::to_string,
        ),
    ]
}

/// Tableau des messages trouvés (boîte, UID, message local) et mention d'une liste tronquée.
pub fn server_search_text(result: &ServerSearchResult) -> String {
    let rows: Vec<Vec<String>> = result.hits.iter().map(hit_row).collect();
    let table = render_table(&["BOÎTE", "UID", "MESSAGE LOCAL"], &rows);
    if result.truncated {
        format!("{table}\n(résultat tronqué : affinez la recherche)")
    } else {
        table
    }
}
