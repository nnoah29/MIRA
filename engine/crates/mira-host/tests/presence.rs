#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Présence des clients : quelqu'un regarde-t-il MIRA maintenant ? Indépendante des notifications.

use mira_host::{Host, Presence};

/// Sans client, personne ne regarde.
#[test]
fn no_client_is_inactive() {
    assert!(!Presence::new().is_active());
}

/// Un client au premier plan suffit à rendre MIRA actif, et le retirer le désactive.
#[test]
fn one_focused_client_activates_and_removal_deactivates() {
    let presence = Presence::new();
    presence.set("a", true);
    assert!(presence.is_active());
    presence.remove("a");
    assert!(!presence.is_active());
}

/// Un client en arrière-plan n'active rien ; deux clients : il suffit qu'un seul soit au premier plan.
#[test]
fn only_a_focused_client_counts_among_several() {
    let presence = Presence::new();
    presence.set("a", false);
    assert!(!presence.is_active());
    presence.set("b", true);
    assert!(presence.is_active());
    presence.set("b", false);
    assert!(!presence.is_active());
}

/// Un client qui repasse au premier plan remplace son état au lieu de s'ajouter.
#[test]
fn setting_the_same_client_twice_keeps_one_entry() {
    let presence = Presence::new();
    presence.set("a", true);
    presence.set("a", true);
    presence.remove("a");
    assert!(!presence.is_active());
}

/// Retirer un client inconnu est sans effet.
#[test]
fn removing_an_unknown_client_is_harmless() {
    let presence = Presence::new();
    presence.set("a", true);
    presence.remove("inconnu");
    assert!(presence.is_active());
}

/// L'hôte porte la présence de ses clients.
#[test]
fn the_host_exposes_its_presence() {
    let host = Host::new();
    host.presence().set("c", true);
    assert!(host.presence().is_active());
}
