//! Les clés d'idempotence portent le nonce de session et ne se répètent jamais.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use action_fixtures::{state_with_mailboxes, state_with_nonce};
use mira_tui::commands::Command;
use mira_tui::state::AppState;
use mira_tui::update::Intent;
use navigation_fixtures::press;
use std::collections::HashSet;

/// Texte de débogage de la requête, qui contient la clé d'idempotence.
fn rendered(commands: &[Command]) -> String {
    format!("{commands:?}")
}

/// Séquence identique sur deux états de nonces différents : aucune clé en commun.
#[test]
fn different_nonces_give_different_keys_for_same_sequence() {
    let mut first = state_with_mailboxes();
    let mut second = state_with_nonce(2);
    let one = rendered(&press(&mut first, Intent::Archive));
    let two = rendered(&press(&mut second, Intent::Archive));
    assert!(one.contains("tui-1-act-1"), "{one}");
    assert!(two.contains("tui-2-act-1"), "{two}");
    assert_ne!(one, two);
}

/// Un même état ne fabrique jamais deux fois la même clé, marquage lu compris.
#[test]
fn same_state_never_repeats_a_key() {
    let mut state = state_with_mailboxes();
    let mut seen = HashSet::new();
    for _ in 0..20 {
        assert!(seen.insert(state.fresh_key("act")));
        assert!(seen.insert(state.fresh_key("read-m1")));
    }
}

/// Le nonce est un paramètre obligatoire de l'état, repris dans le préfixe des clés.
#[test]
fn nonce_is_part_of_every_key() {
    let mut state = AppState::new(80, 24, 424_242);
    assert!(state.fresh_key("x").0.contains("424242"));
}
