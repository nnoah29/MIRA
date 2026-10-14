//! Rendu humain des boîtes d'un compte.

use mira_assembly::mira_vocabulary::Mailbox;

use super::label::json_label;
use super::table::render_table;

/// Cellules d'une ligne de boîte.
fn mailbox_row(mailbox: &Mailbox) -> Vec<String> {
    vec![
        mailbox.id.to_string(),
        mailbox.display_name.clone(),
        json_label(&mailbox.role),
        if mailbox.sync_enabled { "oui" } else { "non" }.to_string(),
        mailbox.unread.to_string(),
        mailbox.total.to_string(),
    ]
}

/// Tableau des boîtes : identifiant, nom, rôle, synchronisée, non lus, total.
pub fn mailboxes_table(mailboxes: &[Mailbox]) -> String {
    let rows: Vec<Vec<String>> = mailboxes.iter().map(mailbox_row).collect();
    render_table(&["ID", "NOM", "RÔLE", "SYNC", "NON LUS", "TOTAL"], &rows)
}
