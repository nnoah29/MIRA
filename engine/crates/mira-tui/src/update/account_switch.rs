//! Changement de compte sélectionné et rechargement de ses boîtes.

use mira_assembly::api::{MailboxRequest, Request};
use mira_assembly::mira_vocabulary::{Account, AccountId};

use crate::commands::{Command, Tag};
use crate::state::{AppState, Selection};

/// Commande de lecture des boîtes d'un compte.
pub fn list_mailboxes(account: &Account) -> Command {
    Command::Execute {
        tag: Tag::Mailboxes(account.id.clone()),
        request: Request::Mailbox(MailboxRequest::ListMailboxes {
            account: account.id.clone(),
        }),
    }
}

/// Identifiant du compte sélectionné.
pub fn selected_id(state: &AppState) -> Option<AccountId> {
    state.accounts.selected().map(|account| account.id.clone())
}

/// Vide les boîtes (et ferme la vue « file ») puis recharge celles du compte sélectionné si ce compte a changé.
pub fn reload_if_changed(state: &mut AppState, previous: Option<AccountId>) -> Vec<Command> {
    if selected_id(state) == previous {
        return Vec::new();
    }
    state.search.cancel();
    state.queue.close();
    state.mailboxes.replace_items(Vec::new());
    state.clear_view();
    match state.accounts.selected() {
        Some(account) => vec![list_mailboxes(account)],
        None => Vec::new(),
    }
}

/// Déplace la sélection de compte puis recharge les boîtes si elle a changé.
fn move_account(state: &mut AppState, step: impl FnOnce(&mut Selection, usize)) -> Vec<Command> {
    let previous = selected_id(state);
    let len = state.accounts.len();
    step(&mut state.accounts.selection, len);
    reload_if_changed(state, previous)
}

/// Passe au compte suivant.
pub fn next_account(state: &mut AppState) -> Vec<Command> {
    move_account(state, Selection::down)
}

/// Passe au compte précédent.
pub fn prev_account(state: &mut AppState) -> Vec<Command> {
    move_account(state, Selection::up)
}
