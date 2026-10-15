//! Lectures à refaire après une modification réussie.

use mira_assembly::api::{AccountRequest, Request};

use super::identity_actions::reload_identities;
use super::maintenance::load_stats;
use crate::commands::{Command, Tag};
use crate::state::AppState;
use crate::state::settings::{FollowUp, SettingsTab};

/// Relit ce que la modification a pu changer ; une page qui n'est plus affichée n'est pas relue.
pub fn follow_up(state: &mut AppState, after: FollowUp) -> Vec<Command> {
    let shown = state.settings.is_open().then_some(state.settings.tab);
    match (after, shown) {
        (FollowUp::Accounts, _) => reload_accounts(state),
        (FollowUp::Identities, Some(SettingsTab::Identities)) => reload_identities(state),
        (FollowUp::Stats, Some(SettingsTab::Maintenance)) => load_stats(state),
        (FollowUp::Identities | FollowUp::Stats | FollowUp::Nothing, _) => Vec::new(),
    }
}

/// Relit les comptes, sauf si une relecture est déjà en cours.
fn reload_accounts(state: &mut AppState) -> Vec<Command> {
    if state.refresh.accounts {
        return Vec::new();
    }
    state.refresh.accounts = true;
    vec![Command::Execute {
        tag: Tag::Accounts,
        request: Request::Account(AccountRequest::ListAccounts),
    }]
}
