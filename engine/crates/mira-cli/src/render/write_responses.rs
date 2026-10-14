//! Rendu humain des réponses d'écriture : identifiants d'actions, brouillons, envoi, file.

use mira_assembly::api::WriteResponse;
use mira_assembly::mira_vocabulary::ActionId;

use super::drafts::{draft_text, drafts_table};
use super::queue::{outbox_table, queue_table};

/// Une ligne par action créée.
fn action_lines(ids: &[ActionId]) -> String {
    let lines: Vec<String> = ids.iter().map(|id| format!("action : {id}")).collect();
    lines.join("\n")
}

/// Texte d'une réponse d'écriture, un `match` exhaustif sur toutes ses formes.
pub fn write_text(response: &WriteResponse) -> String {
    match response {
        WriteResponse::Action(id) => format!("action : {id}"),
        WriteResponse::Actions(ids) => action_lines(ids),
        WriteResponse::Draft(draft) => draft_text(draft),
        WriteResponse::Drafts(drafts) => drafts_table(drafts),
        WriteResponse::Outbox(entries) => outbox_table(entries),
        WriteResponse::QueuedActions(actions) => queue_table(actions),
    }
}
