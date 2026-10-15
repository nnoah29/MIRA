//! Tests du rendu de la vue Paramètres, de ses onglets et de la palette.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod sidebar_fixtures;

use mira_assembly::api::Response;
use mira_assembly::mira_vocabulary::{AccountId, Event, EventKind, EventSeq, Timestamp};
use mira_tui::commands::{SettingsTag, Tag};
use mira_tui::state::AppState;
use mira_tui::state::settings::SettingsTab;
use mira_tui::update::palette::intent::PaletteIntent;
use mira_tui::update::settings::SettingsIntent;
use mira_tui::update::{Intent, Msg, Reply, update};
use sidebar_fixtures::{loaded_state, rendered_text};

/// La barre d'onglets présente les quatre sections des paramètres.
#[test]
fn settings_tab_bar_shows_all_tabs() {
    let mut state = AppState::new(100, 30, 1);
    update(
        &mut state,
        Msg::Intent(Intent::Settings(SettingsIntent::Show(
            SettingsTab::Accounts,
        ))),
    );
    let text = rendered_text(&state);
    assert!(text.contains("Comptes") && text.contains("Identités"));
    assert!(text.contains("Maintenance") && text.contains("Événements"));
}

/// L'onglet Comptes détaille le compte sélectionné et rappelle ses raccourcis.
#[test]
fn settings_accounts_tab_shows_account_details() {
    let mut state = loaded_state();
    update(
        &mut state,
        Msg::Intent(Intent::Settings(SettingsIntent::Show(
            SettingsTab::Accounts,
        ))),
    );
    let text = rendered_text(&state);
    assert!(text.contains("a1@example.org") && text.contains("connecté"));
}

/// L'onglet Identités s'affiche avec son titre et ses indications.
#[test]
fn settings_identities_tab_renders() {
    let mut state = loaded_state();
    update(
        &mut state,
        Msg::Intent(Intent::Settings(SettingsIntent::Show(
            SettingsTab::Identities,
        ))),
    );
    assert!(rendered_text(&state).contains("Identités"));
}

/// L'onglet Maintenance liste les actions disponibles et leurs touches.
#[test]
fn settings_maintenance_tab_renders_actions() {
    let mut state = AppState::new(100, 30, 1);
    update(
        &mut state,
        Msg::Intent(Intent::Settings(SettingsIntent::Show(
            SettingsTab::Maintenance,
        ))),
    );
    let text = rendered_text(&state);
    assert!(text.contains("vider le cache") && text.contains("reconstruire"));
}

/// L'onglet Événements affiche les entrées reçues dans son tableau.
#[test]
fn settings_events_tab_renders_events() {
    let mut state = AppState::new(100, 30, 1);
    let commands = update(
        &mut state,
        Msg::Intent(Intent::Settings(SettingsIntent::Show(SettingsTab::Events))),
    );
    let token = match &commands[0] {
        mira_tui::commands::Command::Execute {
            tag: Tag::Settings(SettingsTag::Events(t)),
            ..
        } => *t,
        _ => panic!("Events attendu"),
    };
    let event = Event {
        seq: EventSeq(42),
        at: Timestamp::from_millis(1000),
        kind: EventKind::AccountAdded {
            account: AccountId::from_raw("a1"),
        },
    };
    let reply = Reply {
        tag: Tag::Settings(SettingsTag::Events(token)),
        result: Ok(Response::Events(vec![event])),
    };
    update(&mut state, Msg::Reply(reply));
    let text = rendered_text(&state);
    assert!(text.contains("42") && text.contains("AccountAdded"));
}

/// La palette de commandes affiche le texte tapé et la liste des commandes.
#[test]
fn palette_popup_renders_typed_text_and_hint() {
    let mut state = AppState::new(100, 30, 1);
    state.palette.open();
    for c in "settings".chars() {
        update(
            &mut state,
            Msg::Intent(Intent::Palette(PaletteIntent::Char(c))),
        );
    }
    let text = rendered_text(&state);
    assert!(text.contains(":settings") && text.contains("Commandes :"));
}
