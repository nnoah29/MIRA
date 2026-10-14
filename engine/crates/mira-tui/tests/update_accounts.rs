//! Rafraîchissement de la liste des comptes et changement de compte.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod sidebar_fixtures;

use mira_tui::update::{Intent, Msg, update};
use sidebar_fixtures::{load_for, loaded_state, refresh};

/// Un rafraîchissement avec le même compte garde les boîtes et ne recharge rien.
#[test]
fn refresh_with_same_account_keeps_mailboxes() {
    let mut state = loaded_state();
    assert!(refresh(&mut state, &["a1", "a2"]).is_empty());
    assert_eq!(state.mailboxes.len(), 1);
}

/// Si le compte sélectionné disparaît, les boîtes sont vidées.
#[test]
fn vanished_selected_account_clears_mailboxes() {
    let mut state = loaded_state();
    assert!(refresh(&mut state, &[]).is_empty());
    assert!(state.mailboxes.is_empty());
}

/// Passer au compte suivant vide les boîtes et charge celles du nouveau compte.
#[test]
fn next_account_clears_and_loads_new_account() {
    let mut state = loaded_state();
    let commands = update(&mut state, Msg::Intent(Intent::NextAccount));
    assert_eq!(state.accounts.selection.selected, Some(1));
    assert!(state.mailboxes.is_empty());
    assert_eq!(commands, vec![load_for("a2")]);
}

/// Revenir au compte précédent recharge le premier compte ; aux bornes, rien ne bouge.
#[test]
fn prev_account_reloads_and_bounds_hold() {
    let mut state = loaded_state();
    assert!(update(&mut state, Msg::Intent(Intent::PrevAccount)).is_empty());
    assert_eq!(state.mailboxes.len(), 1);
    update(&mut state, Msg::Intent(Intent::NextAccount));
    assert!(update(&mut state, Msg::Intent(Intent::NextAccount)).is_empty());
    let commands = update(&mut state, Msg::Intent(Intent::PrevAccount));
    assert_eq!(commands, vec![load_for("a1")]);
}
