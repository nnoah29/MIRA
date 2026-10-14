//! Démarrage d'un brouillon : réponse, réponse à tous, transfert, nouveau message.

use mira_assembly::api::{ComposeRequest, DraftMode, Request};
use mira_assembly::mira_vocabulary::{AccountId, MessageId};

use crate::commands::{Command, Tag};
use crate::state::AppState;
use crate::update::account_switch::selected_id;
use crate::update::action_command::target_message;

/// Demande un brouillon construit depuis le message visé (celui du lecteur s'il a le focus).
fn from_message(state: &mut AppState, mode_of: fn(MessageId) -> DraftMode) -> Vec<Command> {
    let Some(message) = target_message(state) else {
        state.status.message = Some("Aucun message sélectionné".to_string());
        return Vec::new();
    };
    request_draft(state, message.account, mode_of(message.id))
}

/// Répond à l'expéditeur du message visé.
pub fn reply(state: &mut AppState) -> Vec<Command> {
    from_message(state, |message| DraftMode::Reply { message })
}

/// Répond à tous les destinataires du message visé.
pub fn reply_all(state: &mut AppState) -> Vec<Command> {
    from_message(state, |message| DraftMode::ReplyAll { message })
}

/// Transfère le message visé.
pub fn forward(state: &mut AppState) -> Vec<Command> {
    from_message(state, |message| DraftMode::Forward { message })
}

/// Commence un message vide dans le compte sélectionné.
pub fn new_message(state: &mut AppState) -> Vec<Command> {
    let Some(account) = selected_id(state) else {
        state.status.message = Some("Aucun compte sélectionné".to_string());
        return Vec::new();
    };
    request_draft(state, account, DraftMode::New)
}

/// Demande la création ; sans effet si un brouillon est déjà ouvert ou en cours de création.
fn request_draft(state: &mut AppState, account: AccountId, mode: DraftMode) -> Vec<Command> {
    if !state.compose.is_closed() {
        return Vec::new();
    }
    let token = state.fresh_key("draft");
    state.compose = crate::state::ComposeSlot::Creating(token.clone());
    state.status.message = Some("Création du brouillon…".to_string());
    vec![Command::Execute {
        tag: Tag::DraftCreated(token),
        request: Request::Compose(ComposeRequest::CreateDraft {
            account,
            identity: None,
            mode,
        }),
    }]
}
