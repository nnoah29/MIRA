//! Un envoi confirmé prime sur une fermeture (ou une sortie) demandée avant : il n'est jamais perdu.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod send_fixtures;
mod sidebar_fixtures;

use compose_fixtures::{compose, draft, open_modified, saved_reply, type_in_draft};
use mira_tui::commands::Command;
use mira_tui::state::compose::SendPhase;
use mira_tui::state::{AppState, Popup};
use mira_tui::update::{ComposeIntent, Intent, update};
use navigation_fixtures::press;
use send_fixtures::{count_sends, send_command, send_confirmed};

/// Ctrl+s en vol, puis `request` (Échap ou Ctrl+c) et `s` : la fermeture attend l'enregistrement.
fn save_then_close_pending(request: Intent) -> AppState {
    let mut state = open_modified();
    compose(&mut state, ComposeIntent::Save);
    press(&mut state, request);
    assert_eq!(state.popup, Popup::ConfirmCloseDraft);
    assert!(compose(&mut state, ComposeIntent::SaveAndClose).is_empty());
    state
}

/// Ctrl+s → Échap → `s` → F5 → Entrée → réponse : exactement un `SendDraft`, l'éditeur reste jusqu'à la réponse d'envoi.
#[test]
fn confirmed_send_survives_a_pending_save_and_close() {
    let mut state = save_then_close_pending(Intent::Compose(ComposeIntent::Close));
    assert!(send_confirmed(&mut state).is_empty());
    let saved = draft("d1", &["bob@example.org"], "Re: Bonjour!", "Corps");
    let commands = update(&mut state, saved_reply("d1", Ok(saved)));
    assert_eq!(commands, vec![send_command("d1", "tui-1-send-2")]);
    let open = state.compose.open().unwrap();
    assert_eq!(open.send_phase(), SendPhase::Sending);
}

/// Même suite avec Ctrl+c : pas de sortie avant l'envoi ; un Ctrl+c donné ensuite quitte proprement.
#[test]
fn confirmed_send_cancels_an_earlier_quit() {
    let mut state = save_then_close_pending(Intent::Quit);
    send_confirmed(&mut state);
    let saved = draft("d1", &["bob@example.org"], "Re: Bonjour!", "Corps");
    let commands = update(&mut state, saved_reply("d1", Ok(saved)));
    assert_eq!(count_sends(&commands), 1);
    assert!(!commands.contains(&Command::Quit) && !state.should_quit);
    assert_eq!(press(&mut state, Intent::Quit), vec![Command::Quit]);
}

/// Texte tapé entre `s` et F5 : enregistré à son tour, puis l'envoi part ; jamais perdu.
#[test]
fn text_typed_between_save_and_send_is_saved_then_sent() {
    let mut state = save_then_close_pending(Intent::Compose(ComposeIntent::Close));
    type_in_draft(&mut state, "?");
    send_confirmed(&mut state);
    let saved = draft("d1", &["bob@example.org"], "Re: Bonjour!", "Corps");
    let commands = update(&mut state, saved_reply("d1", Ok(saved)));
    assert_eq!(
        (commands.len(), count_sends(&commands)),
        (1, 0),
        "{commands:?}"
    );
    assert!(state.compose.open().is_some());
    let saved = draft("d1", &["bob@example.org"], "Re: Bonjour!?", "Corps");
    let commands = update(&mut state, saved_reply("d1", Ok(saved)));
    assert_eq!(commands, vec![send_command("d1", "tui-1-send-2")]);
}

/// La confirmation d'envoi encore ouverte à l'arrivée de la réponse garde son brouillon : jamais de fenêtre vide.
#[test]
fn send_confirmation_never_outlives_its_draft() {
    let mut state = save_then_close_pending(Intent::Compose(ComposeIntent::Close));
    compose(&mut state, ComposeIntent::Send);
    assert_eq!(state.popup, Popup::ConfirmSend);
    let saved = draft("d1", &["bob@example.org"], "Re: Bonjour!", "Corps");
    update(&mut state, saved_reply("d1", Ok(saved)));
    let still_valid = state.popup != Popup::ConfirmSend || state.compose.open().is_some();
    assert!(still_valid);
    assert!(state.compose.open().is_some());
}
