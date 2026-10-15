//! Tests des transitions d'état des paramètres, comptes, maintenance et journal.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod sidebar_fixtures;

use mira_assembly::api::{AccountRequest, EventRequest, MaintenanceRequest, Request, Response};
use mira_assembly::mira_vocabulary::{AccountId, Event, EventKind, EventSeq, Timestamp};
use mira_tui::commands::{Command, SettingsTag, Tag};
use mira_tui::state::AppState;
use mira_tui::state::settings::{Ask, Dialog, EVENTS_PAGE, SettingsTab};
use mira_tui::update::settings::{AccountOp, FormOp, MaintenanceOp, PageOp, SettingsIntent};
use mira_tui::update::{Intent, Msg, Reply, update};
use sidebar_fixtures::{account, loaded_state};

/// Envoie une intention des paramètres à la boucle de mise à jour.
fn send(state: &mut AppState, intent: SettingsIntent) -> Vec<Command> {
    update(state, Msg::Intent(Intent::Settings(intent)))
}

/// La suppression d'un compte exige de taper son adresse exacte dans le formulaire.
#[test]
fn delete_account_requires_typing_account_name() {
    let mut state = loaded_state();
    send(&mut state, SettingsIntent::Show(SettingsTab::Accounts));
    send(&mut state, SettingsIntent::Account(AccountOp::Delete));
    assert!(matches!(state.settings.dialog, Dialog::Form(_)));
    send(&mut state, SettingsIntent::Form(FormOp::Char('x')));
    assert!(send(&mut state, SettingsIntent::Form(FormOp::Submit)).is_empty());
    assert!(
        state
            .status
            .message
            .as_ref()
            .unwrap()
            .contains("rien n'est supprimé")
    );
    update(&mut state, Msg::Intent(Intent::Cancel));
    send(&mut state, SettingsIntent::Account(AccountOp::Delete));
    for c in "a1@example.org".chars() {
        send(&mut state, SettingsIntent::Form(FormOp::Char(c)));
    }
    let commands = send(&mut state, SettingsIntent::Form(FormOp::Submit));
    let deleted = commands.into_iter().any(|cmd| match cmd {
        Command::Execute {
            request: Request::Account(AccountRequest::DeleteAccount { account }),
            ..
        } => account == AccountId::from_raw("a1"),
        _ => false,
    });
    assert!(deleted);
}

/// Mettre en pause et reprendre un compte émet les requêtes correspondantes vers le moteur.
#[test]
fn pause_and_resume_emit_requests() {
    let mut state = loaded_state();
    send(&mut state, SettingsIntent::Show(SettingsTab::Accounts));
    let pause_cmds = send(&mut state, SettingsIntent::Account(AccountOp::Pause));
    let token = pause_cmds
        .into_iter()
        .find_map(|cmd| match cmd {
            Command::Execute {
                tag: Tag::Settings(SettingsTag::Operation(t)),
                request: Request::Account(AccountRequest::PauseAccount { account }),
            } if account == AccountId::from_raw("a1") => Some(t),
            _ => None,
        })
        .unwrap();
    let reply = Reply {
        tag: Tag::Settings(SettingsTag::Operation(token)),
        result: Ok(Response::Account(account("a1"))),
    };
    update(&mut state, Msg::Reply(reply));
    let resume_cmds = send(&mut state, SettingsIntent::Account(AccountOp::Resume));
    let resumed = resume_cmds.into_iter().any(|cmd| match cmd {
        Command::Execute {
            request: Request::Account(AccountRequest::ResumeAccount { account }),
            ..
        } => account == AccountId::from_raw("a1"),
        _ => false,
    });
    assert!(resumed);
}

/// La modification d'un compte n'envoie que les champs qui ont réellement été changés.
#[test]
fn update_account_sends_only_changed_fields() {
    let mut state = loaded_state();
    send(&mut state, SettingsIntent::Show(SettingsTab::Accounts));
    send(&mut state, SettingsIntent::Account(AccountOp::Edit));
    send(&mut state, SettingsIntent::Form(FormOp::Char('!')));
    let commands = send(&mut state, SettingsIntent::Form(FormOp::Submit));
    let updated = commands.into_iter().any(|cmd| match cmd {
        Command::Execute {
            request: Request::Account(AccountRequest::UpdateAccount { changes, .. }),
            ..
        } => {
            changes.display_name == Some("a1!".to_string())
                && changes.window_days.is_none()
                && changes.servers.is_none()
        }
        _ => false,
    });
    assert!(updated);
}

/// Le journal d'événements pagine avec after positionné sur le dernier événement reçu.
#[rustfmt::skip]
#[test]
fn events_log_pages_with_after_seq() {
    let mut state = AppState::new(100, 30, 1);
    let commands = send(&mut state, SettingsIntent::Show(SettingsTab::Events));
    let token = match &commands[0] {
        Command::Execute { tag: Tag::Settings(SettingsTag::Events(t)), .. } => *t,
        _ => panic!("Events attendu"),
    };
    let make_event = |seq: u32| Event {
        seq: EventSeq(u64::from(seq)),
        at: Timestamp::from_millis(1000),
        kind: EventKind::AccountAdded { account: AccountId::from_raw("a1") },
    };
    let events = (1..=EVENTS_PAGE).map(make_event).collect();
    let reply = Reply { tag: Tag::Settings(SettingsTag::Events(token)), result: Ok(Response::Events(events)) };
    update(&mut state, Msg::Reply(reply));
    let next_cmds = send(&mut state, SettingsIntent::Page(PageOp::Next));
    let paged = next_cmds.into_iter().any(|cmd| match cmd {
        Command::Execute { request: Request::Event(EventRequest::GetEvents { filter, .. }), .. } => {
            filter.after == Some(EventSeq(u64::from(EVENTS_PAGE)))
        }
        _ => false,
    });
    assert!(paged);
}

/// Les actions de maintenance destructives demandent confirmation avant tout envoi.
#[test]
fn maintenance_actions_require_confirmation() {
    let mut state = AppState::new(100, 30, 1);
    send(&mut state, SettingsIntent::Show(SettingsTab::Maintenance));
    send(
        &mut state,
        SettingsIntent::Maintenance(MaintenanceOp::Purge),
    );
    assert_eq!(state.settings.dialog, Dialog::Confirm(Ask::PurgeCache));
    update(&mut state, Msg::Intent(Intent::Cancel));
    assert_eq!(state.settings.dialog, Dialog::None);
    send(
        &mut state,
        SettingsIntent::Maintenance(MaintenanceOp::Rebuild),
    );
    assert_eq!(state.settings.dialog, Dialog::Confirm(Ask::RebuildIndex));
    let commands = update(&mut state, Msg::Intent(Intent::Confirm));
    let confirmed = commands.into_iter().any(|cmd| {
        matches!(
            cmd,
            Command::Execute {
                request: Request::Maintenance(MaintenanceRequest::RebuildSearchIndex),
                ..
            }
        )
    });
    assert!(confirmed);
}
