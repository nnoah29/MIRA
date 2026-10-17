//! Dessin de la vue « file » : états, texte neutralisé, état vide, terminaux minuscules, fenêtre sur 5 000 fiches.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod queue_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use action_fixtures::state_with_mailboxes;
use compose_fixtures::screen;
use mira_assembly::mira_vocabulary::{ActionKind, ActionState};
use mira_tui::update::{Intent, QueueIntent};
use mira_tui::view::queue::queue_rows;
use navigation_fixtures::press;
use queue_fixtures::{open_queue_with, outbox_entry, pending_action};

/// Les envois en attente et échoués sont listés avec leur état, leur objet et l'erreur ; les actions aussi.
#[test]
fn outbox_view_lists_pending_and_failed() {
    let mut state = state_with_mailboxes();
    let waiting = outbox_entry("s1", ActionState::Pending, "Rapport mensuel", None);
    let failed = outbox_entry("s2", ActionState::Failed, "Devis", Some("550 boîte pleine"));
    let archive = pending_action("x3", ActionKind::Archive, ActionState::InProgress);
    open_queue_with(&mut state, vec![waiting, failed], vec![archive]);
    let text = screen(&state, 100, 30).0;
    for wanted in [
        "Rapport mensuel",
        "en attente",
        "Devis",
        "échouée",
        "550 boîte pleine",
        "Archiver",
        "en cours",
    ] {
        assert!(text.contains(wanted), "{wanted} absent :\n{text}");
    }
}

/// File vide : la vue le dit ; pendant la lecture, elle dit qu'elle charge.
#[test]
fn empty_queue_says_so() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Queue(QueueIntent::Open));
    assert!(screen(&state, 100, 30).0.contains("Chargement"));
    let mut state = state_with_mailboxes();
    open_queue_with(&mut state, Vec::new(), Vec::new());
    assert!(screen(&state, 100, 30).0.contains("Aucune action en file"));
}

/// Objet et erreur hostiles (CSI, OSC, inversion de sens) n'atteignent jamais le terminal bruts.
#[test]
fn hostile_queue_text_is_neutralized() {
    let mut state = state_with_mailboxes();
    let subject = "Objet\u{1b}[2J\u{202e}piégé";
    let error = "refus\u{1b}]52;c;cHduZWQ=\u{7}serveur";
    open_queue_with(
        &mut state,
        vec![outbox_entry(
            "s1",
            ActionState::Failed,
            subject,
            Some(error),
        )],
        Vec::new(),
    );
    let text = screen(&state, 120, 30).0;
    assert!(!text.contains('\u{1b}') && !text.contains('\u{202e}') && !text.contains('\u{7}'));
    assert!(
        text.contains("Objetpiégé") && text.contains("refusserveur"),
        "{text}"
    );
}

/// Terminaux minuscules (0×0, 1×1, 10×3) : la vue, avec ou sans fenêtre de décision, ne panique pas.
#[test]
fn queue_view_survives_tiny_terminals() {
    let mut state = state_with_mailboxes();
    let doubtful = outbox_entry("s1", ActionState::Uncertain, "Peut-être", None);
    open_queue_with(&mut state, vec![doubtful], Vec::new());
    for (width, height) in [(0, 0), (1, 1), (10, 3)] {
        screen(&state, width, height);
    }
    press(&mut state, Intent::Queue(QueueIntent::Resolve));
    for (width, height) in [(0, 0), (1, 1), (10, 3)] {
        screen(&state, width, height);
    }
}

/// 5 000 fiches : seules les lignes visibles sont construites, la sélection toujours dedans.
#[test]
fn queue_window_builds_only_visible_rows() {
    let mut state = state_with_mailboxes();
    let many = (0..5_000).map(|i| {
        outbox_entry(
            &format!("s{i}"),
            ActionState::Pending,
            &format!("Objet {i}"),
            None,
        )
    });
    open_queue_with(&mut state, many.collect(), Vec::new());
    press(&mut state, Intent::Bottom);
    let (rows, selected) = queue_rows(&state, 80, 20);
    assert!(rows.len() <= 18, "{}", rows.len());
    assert_eq!(selected, Some(rows.len() - 1));
    assert!(screen(&state, 80, 20).0.contains("Objet 4999"));
}
