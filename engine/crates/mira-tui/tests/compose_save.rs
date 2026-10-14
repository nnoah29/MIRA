//! Enregistrement du brouillon : seuls les champs modifiés partent, jamais une adresse invalide.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use compose_fixtures::{
    compose, draft, edit, focus_on, open_reply, saved_reply, subject_only, type_in_draft,
    update_command,
};
use mira_assembly::api::DraftChanges;
use mira_tui::state::compose::Field;
use mira_tui::update::{ComposeIntent, EditKey, update};
use reader_fixtures::offline_error;

/// Ctrl+s n'envoie que l'objet quand seul l'objet a changé.
#[test]
fn ctrl_s_sends_only_changed_fields() {
    let mut state = open_reply();
    focus_on(&mut state, Field::Subject);
    type_in_draft(&mut state, "!");
    let commands = compose(&mut state, ComposeIntent::Save);
    assert_eq!(
        commands,
        vec![update_command("d1", subject_only("Re: Bonjour!"))]
    );
    assert!(state.compose.open().unwrap().is_saving());
}

/// Rien n'a changé : rien ne part, et la barre d'état le dit.
#[test]
fn save_without_changes_sends_nothing() {
    let mut state = open_reply();
    assert!(compose(&mut state, ComposeIntent::Save).is_empty());
    assert!(!state.compose.open().unwrap().is_saving());
    assert!(state.status.message.is_some());
}

/// Une adresse invalide est signalée dans l'éditeur et n'est jamais envoyée, les autres champs partent.
#[test]
fn invalid_address_is_flagged_not_sent() {
    let mut state = open_reply();
    focus_on(&mut state, Field::To);
    type_in_draft(&mut state, ", pas une adresse");
    assert!(compose(&mut state, ComposeIntent::Save).is_empty());
    let open = state.compose.open().unwrap();
    assert!(open.is_flagged(Field::To) && open.is_dirty(Field::To));
    assert!(
        state
            .status
            .message
            .as_deref()
            .unwrap()
            .contains("pas une adresse")
    );
    focus_on(&mut state, Field::Subject);
    type_in_draft(&mut state, "?");
    let commands = compose(&mut state, ComposeIntent::Save);
    assert_eq!(
        commands,
        vec![update_command("d1", subject_only("Re: Bonjour?"))]
    );
    assert!(state.compose.open().unwrap().is_flagged(Field::To));
}

/// Corriger l'adresse efface le signalement, et l'enregistrement envoie les adresses analysées.
#[test]
fn editing_a_flagged_field_clears_the_flag_and_sends_entries() {
    let mut state = open_reply();
    focus_on(&mut state, Field::To);
    type_in_draft(&mut state, ", zzz");
    compose(&mut state, ComposeIntent::Save);
    for _ in 0..5 {
        edit(&mut state, EditKey::Backspace);
    }
    type_in_draft(&mut state, ", Ann <ann@example.org>");
    assert!(!state.compose.open().unwrap().is_flagged(Field::To));
    let commands = compose(&mut state, ComposeIntent::Save);
    let sent = format!("{commands:?}");
    assert!(
        sent.contains("ann@example.org") && sent.contains("bob@example.org"),
        "{sent}"
    );
    assert_eq!(commands.len(), 1);
}

/// Après la réponse du moteur, l'enregistrement suivant ne renvoie que ce qui a encore changé.
#[test]
fn second_save_sends_only_what_changed_since_the_first() {
    let mut state = open_reply();
    focus_on(&mut state, Field::Subject);
    type_in_draft(&mut state, "!");
    compose(&mut state, ComposeIntent::Save);
    let saved = draft("d1", &["bob@example.org"], "Re: Bonjour!", "Corps");
    update(&mut state, saved_reply("d1", Ok(saved)));
    assert!(!state.compose.open().unwrap().has_unsaved());
    focus_on(&mut state, Field::Body);
    type_in_draft(&mut state, "x");
    let commands = compose(&mut state, ComposeIntent::Save);
    let body = DraftChanges {
        body_text: Some("xCorps".to_string()),
        ..DraftChanges::default()
    };
    assert_eq!(commands, vec![update_command("d1", body)]);
}

/// Un échec garde les modifications à enregistrer, montre l'erreur, et un nouvel essai renvoie le champ.
#[test]
fn failed_save_keeps_changes_and_can_be_retried() {
    let mut state = open_reply();
    focus_on(&mut state, Field::Subject);
    type_in_draft(&mut state, "!");
    compose(&mut state, ComposeIntent::Save);
    update(&mut state, saved_reply("d1", Err(offline_error())));
    let open = state.compose.open().unwrap();
    assert!(open.is_dirty(Field::Subject) && !open.is_saving());
    assert!(state.status.message.is_some());
    let commands = compose(&mut state, ComposeIntent::Save);
    assert_eq!(
        commands,
        vec![update_command("d1", subject_only("Re: Bonjour!"))]
    );
}
