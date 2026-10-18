//! Routage des intentions de rédaction, hors édition (démarrage) puis pendant l'édition.

use super::created::cancel_creation;
use super::intent::ComposeIntent;
use super::{close, edit, fields, save, start};
use crate::commands::Command;
use crate::state::AppState;
use crate::state::compose::SendPhase;
use crate::update::send;
use crate::update::send_dispatch::abandon_pending_send;
use crate::update::{Intent, panes};

/// Route une intention de rédaction reçue hors édition : seuls les démarrages agissent.
pub fn route_idle(state: &mut AppState, intent: ComposeIntent) -> Vec<Command> {
    match intent {
        ComposeIntent::Reply => start::reply(state),
        ComposeIntent::ReplyAll => start::reply_all(state),
        ComposeIntent::Forward => start::forward(state),
        ComposeIntent::New => start::new_message(state),
        ComposeIntent::Edit(_)
        | ComposeIntent::NextField
        | ComposeIntent::PrevField
        | ComposeIntent::Save
        | ComposeIntent::Close
        | ComposeIntent::SaveAndClose
        | ComposeIntent::AskDiscard
        | ComposeIntent::Discard
        | ComposeIntent::Send => Vec::new(),
    }
}

/// Pendant l'édition, l'éditeur reçoit toutes les intentions ; seules la sienne et la sortie agissent.
#[rustfmt::skip]
pub fn route_editing(state: &mut AppState, intent: Intent) -> Vec<Command> {
    match intent {
        Intent::Compose(compose) => on_compose(state, compose),
        Intent::Quit => {
            abandon_pending_send(state);
            panes::quit(state)
        }
        Intent::MoveDown | Intent::MoveUp | Intent::Top | Intent::Bottom | Intent::Open | Intent::Back | Intent::NextPane
        | Intent::PrevPane | Intent::NextAccount | Intent::PrevAccount | Intent::Archive | Intent::Trash
        | Intent::DeletePermanently | Intent::ToggleFlag | Intent::ToggleRead | Intent::OpenMovePicker | Intent::Undo
        | Intent::Confirm | Intent::Cancel | Intent::Search(_) | Intent::Queue(_) | Intent::Drafts(_) | Intent::StartSync
        | Intent::Help | Intent::AccountForm(_) | Intent::Palette(_) | Intent::Settings(_) => Vec::new(),
    }
}

/// Pendant la création, l'écran est modal : seuls Échap (annuler) et la sortie agissent, le reste est ignoré.
#[rustfmt::skip]
pub fn route_creating(state: &mut AppState, intent: Intent) -> Vec<Command> {
    match intent {
        Intent::Compose(ComposeIntent::Close) => cancel_creation(state),
        Intent::Quit => panes::quit(state),
        Intent::Compose(_) | Intent::MoveDown | Intent::MoveUp | Intent::Top | Intent::Bottom | Intent::Open | Intent::Back
        | Intent::NextPane | Intent::PrevPane | Intent::NextAccount | Intent::PrevAccount | Intent::Archive | Intent::Trash
        | Intent::DeletePermanently | Intent::ToggleFlag | Intent::ToggleRead | Intent::OpenMovePicker | Intent::Undo
        | Intent::Confirm | Intent::Cancel | Intent::Search(_) | Intent::Queue(_) | Intent::Drafts(_) | Intent::StartSync
        | Intent::Help | Intent::AccountForm(_) | Intent::Palette(_) | Intent::Settings(_) => Vec::new(),
    }
}

/// Applique une intention de rédaction au brouillon ouvert.
fn on_compose(state: &mut AppState, intent: ComposeIntent) -> Vec<Command> {
    if locked_while_sending(intent) && is_sending(state) {
        return send::busy(state);
    }
    match intent {
        ComposeIntent::Edit(key) => edit::on_edit(state, key),
        ComposeIntent::NextField => fields::next_field(state),
        ComposeIntent::PrevField => fields::previous_field(state),
        ComposeIntent::Save => save::save(state),
        ComposeIntent::Close => close::request_close(state),
        ComposeIntent::Send => send::request_send(state),
        ComposeIntent::Reply
        | ComposeIntent::ReplyAll
        | ComposeIntent::Forward
        | ComposeIntent::New
        | ComposeIntent::SaveAndClose
        | ComposeIntent::AskDiscard
        | ComposeIntent::Discard => Vec::new(),
    }
}

/// Vrai pour les intentions qui changeraient le brouillon ou lanceraient un autre envoi.
fn locked_while_sending(intent: ComposeIntent) -> bool {
    match intent {
        ComposeIntent::Edit(_)
        | ComposeIntent::Save
        | ComposeIntent::Close
        | ComposeIntent::Send => true,
        ComposeIntent::NextField
        | ComposeIntent::PrevField
        | ComposeIntent::Reply
        | ComposeIntent::ReplyAll
        | ComposeIntent::Forward
        | ComposeIntent::New
        | ComposeIntent::SaveAndClose
        | ComposeIntent::AskDiscard
        | ComposeIntent::Discard => false,
    }
}

/// Vrai tant qu'un envoi du brouillon ouvert est engagé (enregistrement préalable ou requête en vol).
fn is_sending(state: &AppState) -> bool {
    state
        .compose
        .open()
        .is_some_and(|compose| compose.send_phase() != SendPhase::Idle)
}
