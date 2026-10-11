//! Aiguillage des commandes vers leur gestionnaire.

use super::kind::{AnyCommand, AuthCommand, CommandKind, SelectedCommand};
use super::{
    append, authenticate, capability, copy, enable, expunge, fetch, id, idle, list, login,
};
use super::{manage, move_cmd, search, select, status, store_cmd};
use crate::imap::parser::command::Command;
use crate::imap::response::error::{HandlerResult, ImapError};
use crate::imap::response::reply::Reply;
use crate::imap::session::state::SessionState;

/// Exécute la commande et transforme un échec en réponse étiquetée
pub(crate) fn dispatch(state: &mut SessionState, command: &Command) -> Reply {
    let result = match CommandKind::parse(&command.name, command.uid) {
        CommandKind::Any(kind) => route_any(state, command, kind),
        CommandKind::Auth(kind) => state
            .require_user()
            .and_then(|_| route_auth(state, command, kind)),
        CommandKind::Selected(kind) => state
            .require_user()
            .and_then(|_| state.require_selected().map(|_| ()))
            .and_then(|()| route_selected(state, command, kind)),
        CommandKind::Unknown => Err(ImapError::bad("Command not recognized")),
    };
    result.unwrap_or_else(|error| error.into_reply(&command.tag))
}

/// Commandes valables dans tous les états
fn route_any(state: &mut SessionState, command: &Command, kind: AnyCommand) -> HandlerResult {
    match kind {
        AnyCommand::Capability => capability::handle_capability(state, command),
        AnyCommand::Noop => capability::handle_noop(state, command),
        AnyCommand::Logout => capability::handle_logout(state, command),
        AnyCommand::Id => id::handle_id(state, command),
        AnyCommand::Login => login::handle_login(state, command),
        AnyCommand::Authenticate => authenticate::handle_authenticate(state, command),
    }
}

/// Commandes de l'état authentifié
fn route_auth(state: &mut SessionState, command: &Command, kind: AuthCommand) -> HandlerResult {
    match kind {
        AuthCommand::Enable => enable::handle_enable(state, command),
        AuthCommand::List => list::handle_list(state, command),
        AuthCommand::Status => status::handle_status(state, command),
        AuthCommand::Create => manage::handle_create(state, command),
        AuthCommand::Delete => manage::handle_delete(state, command),
        AuthCommand::Rename => manage::handle_rename(state, command),
        AuthCommand::Select => select::handle_select(state, command),
        AuthCommand::Examine => select::handle_examine(state, command),
        AuthCommand::Append => append::handle_append(state, command),
        AuthCommand::Idle => idle::handle_idle(state, command),
    }
}

/// Commandes de l'état sélectionné
fn route_selected(
    state: &mut SessionState,
    command: &Command,
    kind: SelectedCommand,
) -> HandlerResult {
    match kind {
        SelectedCommand::Fetch => fetch::handle_fetch(state, command),
        SelectedCommand::Search => search::handle_search(state, command),
        SelectedCommand::Store => store_cmd::handle_store(state, command),
        SelectedCommand::Copy => copy::handle_copy(state, command),
        SelectedCommand::Move => move_cmd::handle_move(state, command),
        SelectedCommand::Expunge => expunge::handle_expunge(state, command),
        SelectedCommand::Close => expunge::handle_close(state, command),
        SelectedCommand::Check => capability::handle_noop(state, command),
    }
}
