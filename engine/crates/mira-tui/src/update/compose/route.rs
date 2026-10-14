//! Routage des intentions de rédaction, hors édition (démarrage) puis pendant l'édition.

use super::intent::ComposeIntent;
use super::{close, edit, fields, save, start};
use crate::commands::Command;
use crate::state::AppState;
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
        | ComposeIntent::Discard => Vec::new(),
    }
}

/// Pendant l'édition, l'éditeur reçoit toutes les intentions ; seules la sienne et la sortie agissent.
#[rustfmt::skip]
pub fn route_editing(state: &mut AppState, intent: Intent) -> Vec<Command> {
    match intent {
        Intent::Compose(compose) => on_compose(state, compose),
        Intent::Quit => panes::quit(state),
        Intent::MoveDown | Intent::MoveUp | Intent::Top | Intent::Bottom | Intent::Open | Intent::Back | Intent::NextPane
        | Intent::PrevPane | Intent::NextAccount | Intent::PrevAccount | Intent::Archive | Intent::Trash
        | Intent::DeletePermanently | Intent::ToggleFlag | Intent::ToggleRead | Intent::OpenMovePicker | Intent::Undo
        | Intent::Confirm | Intent::Cancel | Intent::Search(_) | Intent::StartSync | Intent::Help => Vec::new(),
    }
}

/// Applique une intention de rédaction au brouillon ouvert.
fn on_compose(state: &mut AppState, intent: ComposeIntent) -> Vec<Command> {
    match intent {
        ComposeIntent::Edit(key) => edit::on_edit(state, key),
        ComposeIntent::NextField => fields::next_field(state),
        ComposeIntent::PrevField => fields::previous_field(state),
        ComposeIntent::Save => save::save(state),
        ComposeIntent::Close => close::request_close(state),
        ComposeIntent::Reply
        | ComposeIntent::ReplyAll
        | ComposeIntent::Forward
        | ComposeIntent::New
        | ComposeIntent::SaveAndClose
        | ComposeIntent::Discard => Vec::new(),
    }
}
