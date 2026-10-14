//! Traitement d'un événement du moteur : rechargements déduits de sa nature, sans rafale.

use mira_assembly::api::{AccountRequest, Request};
use mira_assembly::mira_vocabulary::{Event, EventKind};

use super::account_switch::{list_mailboxes, selected_id};
use super::event_reaction::{Reaction, reaction_of};
use super::message_refresh::refresh_view;
use super::sync_status::refresh_status;
use crate::commands::{Command, Tag};
use crate::state::AppState;
use crate::view::safe_text::display_safe;

/// Applique l'événement : les rechargements pertinents partent, ceux déjà en cours ne sont pas redemandés.
pub fn on_event(state: &mut AppState, event: Event) -> Vec<Command> {
    let mut reaction = reaction_of(&event.kind);
    if !concerns_selected_account(state, &event.kind) {
        reaction.content = false;
        reaction.mailboxes = false;
    }
    if let EventKind::ActionFailed { reason, .. } = &event.kind {
        state.status.message = Some(display_safe(reason));
    }
    reload(state, reaction)
}

/// Vrai pour un événement du compte sélectionné ou sans compte.
fn concerns_selected_account(state: &AppState, kind: &EventKind) -> bool {
    match (kind.account(), selected_id(state)) {
        (Some(account), Some(selected)) => *account == selected,
        (Some(_), None) => false,
        (None, _) => true,
    }
}

/// Émet les commandes de rechargement demandées par la réaction.
fn reload(state: &mut AppState, reaction: Reaction) -> Vec<Command> {
    let mut commands = Vec::new();
    if reaction.content {
        commands.extend(refresh_view(state));
    }
    if reaction.mailboxes {
        commands.extend(refresh_mailboxes(state));
    }
    if reaction.accounts && !state.refresh.accounts {
        state.refresh.accounts = true;
        commands.push(Command::Execute {
            tag: Tag::Accounts,
            request: Request::Account(AccountRequest::ListAccounts),
        });
    }
    if reaction.status {
        commands.extend(refresh_status(state));
    }
    commands
}

/// Relit les boîtes du compte sélectionné, sauf si une relecture est déjà en cours.
fn refresh_mailboxes(state: &mut AppState) -> Vec<Command> {
    let Some(account) = state.accounts.selected().cloned() else {
        return Vec::new();
    };
    if state.refresh.mailboxes {
        return Vec::new();
    }
    state.refresh.mailboxes = true;
    vec![list_mailboxes(&account)]
}
