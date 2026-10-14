//! Rendu humain des pièces jointes d'un message.

use mira_assembly::mira_vocabulary::Attachment;

use super::label::json_label;
use super::table::render_table;

/// Cellules d'une ligne de pièce jointe.
fn attachment_row(attachment: &Attachment) -> Vec<String> {
    vec![
        attachment.id.to_string(),
        attachment
            .filename
            .clone()
            .unwrap_or_else(|| "(sans nom)".to_string()),
        attachment.mime_type.clone(),
        attachment.size.to_string(),
        json_label(&attachment.cache),
    ]
}

/// Tableau des pièces jointes : identifiant, nom, type, taille, cache local.
pub fn attachments_table(attachments: &[Attachment]) -> String {
    let rows: Vec<Vec<String>> = attachments.iter().map(attachment_row).collect();
    render_table(&["ID", "NOM", "TYPE", "TAILLE", "CACHE"], &rows)
}
