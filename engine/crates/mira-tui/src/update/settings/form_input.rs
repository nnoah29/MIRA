//! Saisie dans le formulaire ouvert et validation selon ce qu'il modifie.

use super::account_submit::{submit_delete_account, submit_edit_account};
use super::export_submit::submit_export;
use super::identity_submit::{submit_add_identity, submit_edit_identity};
use super::intent::FormOp;
use crate::commands::Command;
use crate::state::AppState;
use crate::state::settings::{Dialog, FormKind};

/// Applique une touche au formulaire ouvert ; sans formulaire, rien ne se passe.
pub fn on_form_op(state: &mut AppState, op: FormOp) -> Vec<Command> {
    let Dialog::Form(form) = &mut state.settings.dialog else {
        return Vec::new();
    };
    match op {
        FormOp::Char(c) => form.push(c),
        FormOp::Backspace => form.pop(),
        FormOp::NextField => form.focus_next(),
        FormOp::PrevField => form.focus_previous(),
        FormOp::Submit => return submit_form(state),
    }
    Vec::new()
}

/// Valide le formulaire ouvert selon ce qu'il modifie ; il ne se ferme que si la demande part.
fn submit_form(state: &mut AppState) -> Vec<Command> {
    let Dialog::Form(form) = state.settings.dialog.clone() else {
        return Vec::new();
    };
    match form.kind.clone() {
        FormKind::EditAccount(id) => submit_edit_account(state, &form, id),
        FormKind::DeleteAccount { account, address } => {
            submit_delete_account(state, &form, account, &address)
        }
        FormKind::AddIdentity(account) => submit_add_identity(state, &form, account),
        FormKind::EditIdentity(identity) => submit_edit_identity(state, &form, identity),
        FormKind::Export(kind) => submit_export(state, &form, kind),
    }
}
