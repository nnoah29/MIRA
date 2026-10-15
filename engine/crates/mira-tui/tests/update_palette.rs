//! Tests de la palette de commandes `:` et de son analyseur.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use mira_tui::input::intent_for;
use mira_tui::state::AppState;
use mira_tui::state::settings::SettingsTab;
use mira_tui::update::palette::parse_command;
use mira_tui::update::settings::SettingsIntent;
use mira_tui::update::{DraftsIntent, Intent, Msg, PaletteIntent, QueueIntent, update};

/// La palette reconnaît les commandes avec ou sans deux-points et refuse les mots inconnus.
#[rustfmt::skip]
#[test]
fn palette_parses_known_commands_and_rejects_unknown() {
    let settings = |tab| Some(Intent::Settings(SettingsIntent::Show(tab)));
    let cases = [
        (":queue", Some(Intent::Queue(QueueIntent::Open))),
        ("queue", Some(Intent::Queue(QueueIntent::Open))),
        (":drafts", Some(Intent::Drafts(DraftsIntent::Open))),
        (":sync", Some(Intent::StartSync)),
        (":help", Some(Intent::Help)),
        (":settings", settings(SettingsTab::Accounts)),
        (":identities", settings(SettingsTab::Identities)),
        (":maintenance", settings(SettingsTab::Maintenance)),
        (":events", settings(SettingsTab::Events)),
        (":inconnu", None),
        ("inconnu", None),
        ("", None),
    ];
    for (input, expected) in cases {
        assert_eq!(parse_command(input), expected, "pour {input}");
    }
}

/// La touche deux-points ouvre la palette de commandes.
#[test]
fn colon_opens_palette() {
    let mut state = AppState::new(100, 30, 1);
    let key = KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE);
    let intent = intent_for(&state, key).unwrap();
    update(&mut state, Msg::Intent(intent));
    assert!(state.palette.is_open());
}

/// Saisir une commande valide dans la palette et la soumettre déclenche l'action voulue.
#[test]
fn palette_typing_and_submit_triggers_intent() {
    let mut state = AppState::new(100, 30, 1);
    state.palette.open();
    for c in ":settings".chars() {
        update(
            &mut state,
            Msg::Intent(Intent::Palette(PaletteIntent::Char(c))),
        );
    }
    update(
        &mut state,
        Msg::Intent(Intent::Palette(PaletteIntent::Submit)),
    );
    assert!(!state.palette.is_open());
    assert!(state.settings.is_open());
    assert_eq!(state.settings.tab, SettingsTab::Accounts);
}

/// Une commande inconnue ferme la palette et affiche une explication dans la barre d'état.
#[test]
fn palette_unknown_command_shows_status_message() {
    let mut state = AppState::new(100, 30, 1);
    state.palette.open();
    for c in ":nimportequoi".chars() {
        update(
            &mut state,
            Msg::Intent(Intent::Palette(PaletteIntent::Char(c))),
        );
    }
    update(
        &mut state,
        Msg::Intent(Intent::Palette(PaletteIntent::Submit)),
    );
    assert!(!state.palette.is_open());
    let message = state.status.message.unwrap();
    assert!(message.contains("Commande inconnue"));
}

/// Fermer la palette par Échap n'exécute aucune commande.
#[test]
fn palette_escape_closes_without_command() {
    let mut state = AppState::new(100, 30, 1);
    state.palette.open();
    for c in ":sync".chars() {
        update(
            &mut state,
            Msg::Intent(Intent::Palette(PaletteIntent::Char(c))),
        );
    }
    let commands = update(
        &mut state,
        Msg::Intent(Intent::Palette(PaletteIntent::Close)),
    );
    assert!(!state.palette.is_open());
    assert!(commands.is_empty());
}
