//! Page Comptes : modifier, mettre en pause, reprendre, supprimer (après avoir tapé l'adresse).

use mira_assembly::api::{AccountRequest, Request};
use mira_assembly::mira_vocabulary::{Account, AccountId};

use super::account_nav::current_account;
use super::intent::AccountOp;
use super::operation::{say, start_operation};
use crate::commands::Command;
use crate::state::AppState;
use crate::state::settings::{Dialog, Field, Form, FormKind, Operation, SettingsTab};

/// Applique une opération de la page Comptes ; sur une autre page ou sous une fenêtre elle est ignorée.
pub fn on_account_op(state: &mut AppState, op: AccountOp) -> Vec<Command> {
    if state.settings.tab != SettingsTab::Accounts || state.settings.dialog != Dialog::None {
        return Vec::new();
    }
    let Some(account) = current_account(state).cloned() else {
        return say(state, "Aucun compte");
    };
    match op {
        AccountOp::Edit => open_form(state, edit_form(&account)),
        AccountOp::Delete => open_form(state, delete_form(&account)),
        AccountOp::Pause => lifecycle(state, Operation::PauseAccount, account.id),
        AccountOp::Resume => lifecycle(state, Operation::ResumeAccount, account.id),
    }
}

/// Pause ou reprise : le moteur reste seul juge de l'état du compte.
fn lifecycle(state: &mut AppState, operation: Operation, account: AccountId) -> Vec<Command> {
    let request = match operation {
        Operation::PauseAccount => AccountRequest::PauseAccount { account },
        _ => AccountRequest::ResumeAccount { account },
    };
    start_operation(state, operation, Request::Account(request))
}

/// Affiche ce formulaire.
fn open_form(state: &mut AppState, form: Form) -> Vec<Command> {
    state.settings.dialog = Dialog::Form(form);
    Vec::new()
}

/// Formulaire de modification : nom, fenêtre de synchro et, si le compte a des serveurs propres, leurs hôtes et ports.
fn edit_form(account: &Account) -> Form {
    let days = account.sync.window_days.to_string();
    let mut fields = vec![
        Field::new("Nom affiché", &account.display_name),
        Field::new("Jours synchronisés", &days),
    ];
    if let Some(servers) = &account.servers {
        fields.push(Field::new("IMAP hôte", &servers.imap.host));
        fields.push(Field::new("IMAP port", &servers.imap.port.to_string()));
        fields.push(Field::new("SMTP hôte", &servers.smtp.host));
        fields.push(Field::new("SMTP port", &servers.smtp.port.to_string()));
    }
    Form::new(FormKind::EditAccount(account.id.clone()), fields)
}

/// Formulaire de suppression : il faut retaper l'adresse du compte.
fn delete_form(account: &Account) -> Form {
    let kind = FormKind::DeleteAccount {
        account: account.id.clone(),
        address: account.address.as_str().to_string(),
    };
    Form::new(kind, vec![Field::new("Adresse du compte", "")])
}
