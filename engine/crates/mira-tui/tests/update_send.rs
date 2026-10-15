//! Envoi d'un brouillon : confirmation obligatoire, une seule clé par brouillon, rien pendant un envoi en vol.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod send_fixtures;
mod sidebar_fixtures;

use compose_fixtures::{compose, edit, open_reply, type_in_draft};
use mira_tui::state::Popup;
use mira_tui::state::compose::{Field, SendPhase};
use mira_tui::update::{ComposeIntent, EditKey, Intent, update};
use navigation_fixtures::press;
use reader_fixtures::offline_error;
use send_fixtures::{count_sends, failed_send, send_command, send_confirmed};

/// Ctrl+Entrée ouvre la confirmation sans rien envoyer ; Échap ferme sans envoi ; Entrée envoie.
#[test]
fn send_requires_confirmation() {
    let mut state = open_reply();
    assert!(compose(&mut state, ComposeIntent::Send).is_empty());
    assert_eq!(state.popup, Popup::ConfirmSend);
    assert!(press(&mut state, Intent::Cancel).is_empty());
    assert_eq!(state.popup, Popup::Closed);
    assert!(state.compose.open().is_some());
    compose(&mut state, ComposeIntent::Send);
    let commands = press(&mut state, Intent::Confirm);
    assert_eq!(commands, vec![send_command("d1", "tui-1-send-2")]);
    assert_eq!(state.popup, Popup::Closed);
}

/// Un envoi refusé garde le texte et l'erreur ; le renvoi réutilise la même clé, même après une modification.
#[test]
fn second_send_reuses_same_idempotency_key() {
    let mut state = open_reply();
    assert_eq!(
        send_confirmed(&mut state),
        vec![send_command("d1", "tui-1-send-2")]
    );
    update(&mut state, failed_send("d1", "tui-1-send-2"));
    let open = state.compose.open().unwrap();
    assert_eq!(open.send_phase(), SendPhase::Idle);
    assert_eq!(open.text(Field::Body), "Corps");
    let shown = state.status.message.clone().unwrap();
    assert!(shown.contains("Échec de l'envoi"), "{shown}");
    assert_eq!(
        send_confirmed(&mut state),
        vec![send_command("d1", "tui-1-send-2")]
    );
    update(&mut state, failed_send("d1", "tui-1-send-2"));
    type_in_draft(&mut state, "x");
    compose(&mut state, ComposeIntent::Send);
    let saving = press(&mut state, Intent::Confirm);
    assert_eq!((saving.len(), count_sends(&saving)), (1, 0));
    let saved = compose_fixtures::draft("d1", &["bob@example.org"], "Re: Bonjour", "xCorps");
    let sending = update(&mut state, compose_fixtures::saved_reply("d1", Ok(saved)));
    assert_eq!(sending, vec![send_command("d1", "tui-1-send-2")]);
}

/// Pendant l'envoi, Ctrl+Entrée est ignoré (message d'état), la frappe aussi : le contenu ne change plus.
#[test]
fn send_while_in_flight_is_ignored() {
    let mut state = open_reply();
    send_confirmed(&mut state);
    assert!(compose(&mut state, ComposeIntent::Send).is_empty());
    assert_eq!(state.popup, Popup::Closed);
    let shown = state.status.message.clone().unwrap();
    assert!(shown.contains("Envoi en cours"), "{shown}");
    type_in_draft(&mut state, "zz");
    edit(&mut state, EditKey::Backspace);
    assert!(compose(&mut state, ComposeIntent::Save).is_empty());
    assert!(compose(&mut state, ComposeIntent::Close).is_empty());
    let open = state.compose.open().unwrap();
    assert_eq!(open.text(Field::Body), "Corps");
    assert_eq!(open.send_phase(), SendPhase::Sending);
    assert_eq!(state.popup, Popup::Closed);
}

/// Deux Ctrl+Entrée et deux confirmations rapprochées : une seule requête `SendDraft`.
#[test]
fn rapid_double_send_emits_a_single_request() {
    let mut state = open_reply();
    let mut commands = Vec::new();
    commands.extend(compose(&mut state, ComposeIntent::Send));
    commands.extend(compose(&mut state, ComposeIntent::Send));
    commands.extend(press(&mut state, Intent::Confirm));
    commands.extend(press(&mut state, Intent::Confirm));
    commands.extend(compose(&mut state, ComposeIntent::Send));
    commands.extend(press(&mut state, Intent::Confirm));
    assert_eq!(count_sends(&commands), 1, "{commands:?}");
}

/// Sans destinataire valide rien ne part : ni confirmation ni requête, la raison est affichée.
#[test]
fn send_without_valid_recipient_sends_nothing() {
    let blank = compose_fixtures::draft("d1", &[], "Objet", "Corps");
    let mut state = compose_fixtures::open_with(blank);
    assert!(compose(&mut state, ComposeIntent::Send).is_empty());
    assert_eq!(state.popup, Popup::Closed);
    let shown = state.status.message.clone().unwrap();
    assert!(shown.contains("aucun destinataire"), "{shown}");
    let mut state = open_reply();
    compose_fixtures::focus_on(&mut state, Field::Cc);
    type_in_draft(&mut state, "zzz");
    assert!(compose(&mut state, ComposeIntent::Send).is_empty());
    assert_eq!(state.popup, Popup::Closed);
    assert!(state.compose.open().unwrap().is_flagged(Field::Cc));
    let shown = state.status.message.clone().unwrap();
    assert!(shown.contains("zzz") && shown.contains("Cc"), "{shown}");
}

/// Modifications non enregistrées : l'enregistrement part d'abord ; s'il échoue, rien n'est envoyé.
#[test]
fn failed_save_before_send_sends_nothing() {
    let mut state = compose_fixtures::open_modified();
    let commands = send_confirmed(&mut state);
    let subject = compose_fixtures::subject_only("Re: Bonjour!");
    assert_eq!(
        commands,
        vec![compose_fixtures::update_command("d1", subject)]
    );
    assert_eq!(
        state.compose.open().unwrap().send_phase(),
        SendPhase::Saving
    );
    let after = update(
        &mut state,
        compose_fixtures::saved_reply("d1", Err(offline_error())),
    );
    assert_eq!(count_sends(&after), 0);
    let open = state.compose.open().unwrap();
    assert_eq!(open.send_phase(), SendPhase::Idle);
    assert!(open.send_key.is_none());
    assert!(state.status.message.is_some());
}
