//! Réponse du moteur à la liste des boîtes d'un compte.

use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::{AccountId, Mailbox, MiraError};

use super::accounts_reply::{report, unexpected};
use super::message_load::load_selected_mailbox;
use crate::commands::Command;
use crate::state::AppState;
use crate::state::mailbox_order::sort_mailboxes;

/// Range les boîtes triées si elles concernent encore le compte sélectionné.
pub fn on_mailboxes(
    state: &mut AppState,
    account: AccountId,
    result: Result<Response, MiraError>,
) -> Vec<Command> {
    state.refresh.mailboxes = false;
    match result {
        Ok(Response::Mailboxes(mailboxes)) => store_mailboxes(state, &account, mailboxes),
        Ok(_) => unexpected(state),
        Err(error) => report(state, error),
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
        return load_first_if_none_wanted(state);
    }
    Vec::new()
}

/// Charge la boîte sélectionnée tant qu'aucune vue de messages n'est voulue.
fn load_first_if_none_wanted(state: &mut AppState) -> Vec<Command> {
    if state.messages.view().is_some() || state.search.is_active() {
        return Vec::new();
    }
    load_selected_mailbox(state)
}
