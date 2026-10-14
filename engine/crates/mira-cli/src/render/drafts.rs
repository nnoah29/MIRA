//! Rendu humain des brouillons.

use mira_assembly::mira_vocabulary::Draft;

use super::addresses::{address_list, first_party};
use super::instant::format_instant;
use super::label::json_label;
use super::table::render_table;

/// Détail d'un brouillon : champs d'en-tête, pièces jointes et corps.
pub fn draft_text(draft: &Draft) -> String {
    format!(
        "ID      : {}\nCompte  : {}\nIdentité: {}\nÉtat    : {}\nÀ       : {}\nCc      : {}\nCci     : {}\nObjet   : {}\nPièces jointes : {}\n\n{}",
        draft.id,
        draft.account,
        draft.identity,
        json_label(&draft.state),
        address_list(&draft.to),
        address_list(&draft.cc),
        address_list(&draft.bcc),
        draft.subject,
        draft.attachments.len(),
        draft.body_text
    )
}

/// Cellules d'une ligne de brouillon.
fn draft_row(draft: &Draft) -> Vec<String> {
    vec![
        draft.id.to_string(),
        draft.subject.clone(),
        first_party(&draft.to),
        json_label(&draft.state),
        format_instant(draft.updated_at),
    ]
}

/// Tableau des brouillons : identifiant, objet, destinataire, état, dernière modification.
pub fn drafts_table(drafts: &[Draft]) -> String {
    let rows: Vec<Vec<String>> = drafts.iter().map(draft_row).collect();
    render_table(&["ID", "OBJET", "À", "ÉTAT", "MODIFIÉ"], &rows)
}
