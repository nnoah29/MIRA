//! Réponse du moteur à la liste des comptes.

use mira_assembly::api::{MailboxRequest, Request, Response};
use mira_assembly::mira_vocabulary::{Account, MiraError};

use crate::commands::{Command, Tag};
use crate::state::AppState;

/// Range les comptes reçus et demande les boîtes du compte sélectionné.
pub fn on_accounts(state: &mut AppState, result: Result<Response, MiraError>) -> Vec<Command> {
    match result {
        Ok(Response::Accounts(accounts)) => store_accounts(state, accounts),
        Ok(_) => unexpected(state),
        Err(error) => report(state, error),
    }
}

/// Remplace les comptes et charge les boîtes du compte sélectionné s'il y en a un.
fn store_accounts(state: &mut AppState, accounts: Vec<Account>) -> Vec<Command> {
    state.accounts.replace_items(accounts);
    state.mailboxes.replace_items(Vec::new());
    match state.accounts.selected() {
        Some(account) => vec![list_mailboxes(account)],
        None => Vec::new(),
    }
}

/// Commande de lecture des boîtes d'un compte.
fn list_mailboxes(account: &Account) -> Command {
    Command::Execute {
        tag: Tag::Mailboxes(account.id.clone()),
        request: Request::Mailbox(MailboxRequest::ListMailboxes {
            account: account.id.clone(),
        }),
    }
}

/// Montre l'erreur du moteur dans la barre d'état.
fn report(state: &mut AppState, error: MiraError) -> Vec<Command> {
    state.status.message = Some(error.to_string());
    Vec::new()
}

/// Signale une réponse qui ne correspond pas à la requête.
pub fn unexpected(state: &mut AppState) -> Vec<Command> {
    state.status.message = Some("réponse inattendue du moteur".to_string());
    Vec::new()
}
