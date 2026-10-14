//! Réponse du moteur à la liste des boîtes d'un compte.

use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::{AccountId, Mailbox, MiraError};

use super::accounts_reply::unexpected;
use crate::commands::Command;
use crate::state::AppState;
use crate::state::mailbox_order::sort_mailboxes;

/// Range les boîtes triées si elles concernent encore le compte sélectionné.
pub fn on_mailboxes(
    state: &mut AppState,
    account: AccountId,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    match result {
        Ok(Response::Mailboxes(mailboxes)) => store_mailboxes(state, &account, mailboxes),
        Ok(_) => unexpected(state),
        Err(error) => {
            state.status.message = Some(error.to_string());
            Vec::new()
        }
    }
}

/// Remplace les boîtes affichées, sauf si la réponse est périmée.
fn store_mailboxes(
    state: &mut AppState,
    account: &AccountId,
    mut mailboxes: Vec<Mailbox>,
) -> Vec<Command> {
    let current = state.accounts.selected().map(|selected| &selected.id);
    if current == Some(account) {
        sort_mailboxes(&mut mailboxes);
        state.mailboxes.replace_items(mailboxes);
    }
    Vec::new()
}
