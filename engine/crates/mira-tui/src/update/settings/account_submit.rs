//! Validation des formulaires de compte : seuls les champs changés partent, la suppression exige l'adresse.

use mira_assembly::api::{AccountChanges, AccountRequest, Request};
use mira_assembly::mira_vocabulary::{Account, AccountId, Security, ServerEndpoint, ServerSettings};

use super::operation::{say, submit_operation};
use crate::commands::Command;
use crate::state::AppState;
use crate::state::settings::{Dialog, Form, Operation};

/// Valide le formulaire de modification ; une valeur invalide laisse le formulaire ouvert et ne part pas.
pub fn submit_edit_account(state: &mut AppState, form: &Form, id: AccountId) -> Vec<Command> {
    let Some(account) = state.accounts.items().iter().find(|a| a.id == id).cloned() else {
        state.settings.dialog = Dialog::None;
        return say(state, "Ce compte n'existe plus");
    };
    let changes = match changes_from(form, &account) {
        Ok(changes) => changes,
        Err(message) => return say(state, &message),
    };
    if changes == AccountChanges::default() {
        state.settings.dialog = Dialog::None;
        return say(state, "Aucune modification");
    }
    let request = AccountRequest::UpdateAccount {
        account: id,
        changes,
    };
    submit_operation(state, Operation::UpdateAccount, Request::Account(request))
}

/// Valide la suppression : l'adresse tapée doit être celle du compte, sinon rien ne part.
pub fn submit_delete_account(
    state: &mut AppState,
    form: &Form,
    account: AccountId,
    address: &str,
) -> Vec<Command> {
    if !form.value(0).trim().eq_ignore_ascii_case(address) {
        return say(state, "Ce texte n'est pas l'adresse du compte : rien n'est supprimé");
    }
    let request = Request::Account(AccountRequest::DeleteAccount { account });
    submit_operation(state, Operation::DeleteAccount, request)
}

/// Champs changés du formulaire, ou le message qui explique la valeur invalide.
fn changes_from(form: &Form, account: &Account) -> Result<AccountChanges, String> {
    let mut changes = AccountChanges::default();
    if form.changed(0) {
        changes.display_name = Some(form.value(0).trim().to_string());
    }
    if form.changed(1) {
        let days = form.value(1).trim().parse::<u32>();
        changes.window_days = Some(days.map_err(|_| "Nombre de jours invalide".to_string())?);
    }
    changes.servers = changed_servers(form, account)?;
    Ok(changes)
}

/// Serveurs complets si un hôte ou un port a changé (le chiffrement reste celui du compte).
fn changed_servers(form: &Form, account: &Account) -> Result<Option<ServerSettings>, String> {
    let Some(current) = &account.servers else {
        return Ok(None);
    };
    if !(2..6).any(|index| form.changed(index)) {
        return Ok(None);
    }
    Ok(Some(ServerSettings {
        imap: endpoint(form, 2, current.imap.security)?,
        smtp: endpoint(form, 4, current.smtp.security)?,
    }))
}

/// Serveur lu aux champs hôte (`first`) et port (`first + 1`).
fn endpoint(form: &Form, first: usize, security: Security) -> Result<ServerEndpoint, String> {
    let host = form.value(first).trim().to_string();
    let port = form.value(first + 1).trim().parse::<u16>();
    match (host.is_empty(), port) {
        (false, Ok(port)) if port > 0 => Ok(ServerEndpoint {
            host,
            port,
            security,
        }),
        _ => Err("Serveur ou port invalide".to_string()),
    }
}
