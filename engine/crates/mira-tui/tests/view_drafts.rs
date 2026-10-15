//! Dessin de la vue « Brouillons » : fiches, état vide, texte neutralisé, terminaux minuscules, fenêtre sur 5 000.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod drafts_fixtures;
mod input_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use action_fixtures::state_with_mailboxes;
use compose_fixtures::{draft, focus_on, screen, type_in_draft};
use drafts_fixtures::{open_drafts_with, reopen, scheduled, sending};
use input_fixtures::{key, type_keys};
use mira_assembly::mira_vocabulary::AddressEntry;
use mira_tui::state::compose::Field;
use mira_tui::update::{ComposeIntent, DraftsIntent, Intent, update};
use mira_tui::view::drafts::drafts_rows;
use navigation_fixtures::press;

/// Chaque brouillon montre son état, ses destinataires et son objet.
#[test]
fn drafts_view_lists_subject_recipients_state() {
    let mut state = state_with_mailboxes();
    let drafts = vec![
        draft("d1", &["bob@example.org"], "Rapport mensuel", ""),
        scheduled(draft("d2", &["ann@example.org"], "Devis", "")),
        sending(draft("d3", &[], "", "")),
    ];
    open_drafts_with(&mut state, drafts);
    let text = screen(&state, 120, 30).0;
    for wanted in [
        "Brouillons (3)",
        "Rapport mensuel",
        "bob@example.org",
        "Devis",
        "planifié",
        "en cours d'envoi",
        "(sans objet)",
        "(aucun destinataire)",
    ] {
        assert!(text.contains(wanted), "{wanted} absent :\n{text}");
    }
}

/// Pendant la lecture la vue dit qu'elle charge ; sans brouillon elle dit « Aucun brouillon ».
#[test]
fn empty_drafts_view_shows_empty_text() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Drafts(DraftsIntent::Open));
    assert!(screen(&state, 100, 30).0.contains("Chargement"));
    update(&mut state, drafts_fixtures::drafts_reply(1, Vec::new()));
    assert!(screen(&state, 100, 30).0.contains("Aucun brouillon"));
}

/// Objet et nom de destinataire hostiles (liste et confirmation) n'atteignent jamais le terminal bruts.
#[test]
fn drafts_text_goes_through_display_safe() {
    let mut state = state_with_mailboxes();
    let mut hostile = draft("d1", &[], "Objet\u{1b}[2J\u{202e}piégé", "");
    hostile.to = vec![AddressEntry {
        name: Some("Bob\u{1b}]52;c;cHduZWQ=\u{7}Leponge".to_string()),
        address: "bob@example.org".to_string(),
    }];
    open_drafts_with(&mut state, vec![hostile]);
    let text = screen(&state, 140, 30).0;
    assert!(!text.contains('\u{1b}') && !text.contains('\u{202e}') && !text.contains('\u{7}'));
    assert!(
        text.contains("Objetpiégé") && text.contains("BobLeponge"),
        "{text}"
    );
    type_keys(&mut state, &[key('D')]);
    let popup = screen(&state, 140, 30).0;
    assert!(
        popup.contains("Objetpiégé") && !popup.contains('\u{1b}'),
        "{popup}"
    );
}

/// Terminaux minuscules (0×0, 1×1, 10×3) : vue, confirmation de suppression et chargement ne paniquent pas.
#[test]
fn drafts_view_survives_tiny_terminals() {
    let mut state = state_with_mailboxes();
    press(&mut state, Intent::Drafts(DraftsIntent::Open));
    for (width, height) in [(0, 0), (1, 1), (10, 3)] {
        screen(&state, width, height);
    }
    update(
        &mut state,
        drafts_fixtures::drafts_reply(1, drafts_fixtures::named(&["A"])),
    );
    for (width, height) in [(0, 0), (1, 1), (10, 3), (3, 2)] {
        screen(&state, width, height);
    }
    type_keys(&mut state, &[key('D')]);
    for (width, height) in [(0, 0), (1, 1), (10, 3)] {
        screen(&state, width, height);
    }
}

/// 5 000 brouillons : seules les lignes visibles sont construites, la sélection toujours dedans.
#[test]
fn drafts_window_builds_only_visible_rows() {
    let mut state = state_with_mailboxes();
    let many: Vec<_> = (0..5_000)
        .map(|i| draft(&format!("d{i}"), &[], &format!("Objet {i}"), ""))
        .collect();
    open_drafts_with(&mut state, many);
    press(&mut state, Intent::Bottom);
    let (rows, selected) = drafts_rows(&state, 80, 20);
    assert!(rows.len() <= 18, "{}", rows.len());
    assert_eq!(selected, Some(rows.len() - 1));
    assert!(screen(&state, 80, 20).0.contains("Objet 4999"));
}

/// Abandonner un brouillon rouvert : la fenêtre dit que la version enregistrée est gardée.
#[test]
fn reopened_discard_popup_says_saved_version_is_kept() {
    let mut state = reopen(draft("d1", &["bob@example.org"], "Re: Bonjour", "Corps"));
    focus_on(&mut state, Field::Subject);
    type_in_draft(&mut state, "!");
    press(&mut state, Intent::Compose(ComposeIntent::Close));
    press(&mut state, Intent::Compose(ComposeIntent::AskDiscard));
    let text = screen(&state, 100, 30).0;
    assert!(text.contains("version enregistrée"), "{text}");
}
