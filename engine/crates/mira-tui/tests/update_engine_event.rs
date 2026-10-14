//! Événements du moteur : quels rechargements ils déclenchent, sans rafale.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod engine_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod search_fixtures;
mod sidebar_fixtures;

use action_fixtures::{reload_command, reload_reply, state_with_mailboxes};
use engine_fixtures::{
    event, mailboxes_command, status_command, sync_finished, sync_status_command,
};
use mira_assembly::api::{Request, SyncRequest, SyncScope};
use mira_assembly::mira_vocabulary::{AccountId, EventKind};
use mira_tui::commands::{Command, Tag};
use mira_tui::update::{Intent, SearchIntent, update};
use navigation_fixtures::press;
use search_fixtures::type_text;

/// Une synchro terminée recharge la vue courante, les boîtes du compte et l'état.
#[test]
fn sync_event_refreshes_current_view() {
    let mut state = state_with_mailboxes();
    let commands = update(&mut state, sync_finished("a1"));
    assert!(commands.contains(&reload_command()), "{commands:?}");
    assert!(commands.contains(&mailboxes_command("a1")));
    assert!(commands.contains(&status_command()));
    assert!(commands.contains(&sync_status_command()));
    assert_eq!(state.messages.len(), 2);
}

/// Une rafale d'événements n'empile pas de rechargements tant que le premier est en cours.
#[test]
fn burst_of_events_does_not_stack_reloads() {
    let mut state = state_with_mailboxes();
    update(&mut state, sync_finished("a1"));
    let second = update(&mut state, sync_finished("a1"));
    assert!(second.is_empty(), "{second:?}");
    update(&mut state, reload_reply(2));
    assert!(update(&mut state, sync_finished("a1")).contains(&reload_command()));
}

/// Un événement sans effet visible (progression, diagnostic) ne déclenche rien.
#[test]
fn irrelevant_event_triggers_nothing() {
    let mut state = state_with_mailboxes();
    let kind = EventKind::AttachmentReady {
        account: AccountId::from_raw("a1"),
        attachment: mira_assembly::mira_vocabulary::AttachmentId::from_raw("p1"),
        operation: mira_assembly::mira_vocabulary::OperationId::from_raw("o"),
    };
    assert!(update(&mut state, event(kind)).is_empty());
    assert!(update(&mut state, event(EventKind::EngineStarted)).is_empty());
}

/// Pendant une recherche, un événement ne remplace pas les résultats par la boîte.
#[test]
fn event_during_search_keeps_results() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Search(SearchIntent::Open));
    type_text(&mut state, "x");
    press(&mut state, Intent::Search(SearchIntent::Submit));
    let commands = update(&mut state, sync_finished("a1"));
    assert!(!commands.contains(&reload_command()));
    assert!(commands.contains(&status_command()));
}

/// `Ctrl+r` demande la synchronisation de tous les comptes.
#[test]
fn ctrl_r_starts_sync_for_all() {
    let mut state = state_with_mailboxes();
    let commands = press(&mut state, Intent::StartSync);
    assert_eq!(
        commands,
        vec![Command::Execute {
            tag: Tag::StartSync,
            request: Request::Sync(SyncRequest::StartSync {
                scope: SyncScope::All
            }),
        }]
    );
}
