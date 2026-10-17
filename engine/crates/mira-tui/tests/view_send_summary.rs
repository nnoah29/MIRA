//! Confirmation d'envoi résumée : chaque champ compté et ses coupes signalées, objet et touches toujours visibles.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use compose_fixtures::{compose, open_with, screen};
use mira_assembly::mira_vocabulary::{AddressEntry, Draft};
use mira_tui::state::AppState;
use mira_tui::update::ComposeIntent;

/// `count` adresses `<prefix><i>@example.org`.
fn many(prefix: &str, count: usize) -> Vec<AddressEntry> {
    (0..count)
        .map(|i| AddressEntry {
            name: None,
            address: format!("{prefix}{i}@example.org"),
        })
        .collect()
}

/// Éditeur sur ce brouillon, confirmation d'envoi ouverte.
fn confirming(draft: Draft) -> AppState {
    let mut state = open_with(draft);
    compose(&mut state, ComposeIntent::Send);
    state
}

/// 8 adresses en À, ~200 caractères en Cc, 50 en Cci : total juste, coupes signalées, objet et touches visibles.
#[test]
fn crowded_recipients_keep_subject_and_keys_visible() {
    let mut draft = compose_fixtures::draft("d1", &[], "Re: Bonjour", "Corps");
    draft.to = many("destinataire", 8);
    draft.cc = many("copie-tres-longue-adresse-numero-", 5);
    draft.bcc = many("cache", 50);
    let text = screen(&confirming(draft), 100, 30).0;
    assert!(text.contains("63 destinataire(s)"), "{text}");
    assert!(text.contains("(+"), "{text}");
    assert!(text.contains("Cci : 50 destinataire(s)"), "{text}");
    assert!(text.contains("Objet : Re: Bonjour"), "{text}");
    assert!(text.contains("Entrée ou y : envoyer"), "{text}");
    assert!(!text.contains("cache0@example.org"), "{text}");
}

/// Objet de 500 caractères : tronqué avec « … », jamais masqué ni débordant.
#[test]
fn long_subject_is_cut_with_an_ellipsis() {
    let subject = format!("Début{}FIN", "x".repeat(495));
    let draft = compose_fixtures::draft("d1", &["bob@example.org"], &subject, "Corps");
    let text = screen(&confirming(draft), 100, 30).0;
    let line = text.lines().find(|l| l.contains("Objet : Début")).unwrap();
    assert!(line.contains('…') && !line.contains("FIN"), "{line}");
    assert!(text.contains("Entrée ou y : envoyer"), "{text}");
}

/// Résumé chargé sur des terminaux minuscules : aucune panique.
#[test]
fn crowded_summary_survives_tiny_terminals() {
    let mut draft = compose_fixtures::draft("d1", &[], &"o".repeat(500), "Corps");
    draft.to = many("a", 30);
    let state = confirming(draft);
    for (width, height) in [(0, 0), (1, 1), (10, 3), (30, 6)] {
        screen(&state, width, height);
    }
}
