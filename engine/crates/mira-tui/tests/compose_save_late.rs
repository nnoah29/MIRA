//! Enregistrement : réponses tardives, doubles demandes et corps HTML.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod action_fixtures;
mod compose_fixtures;
mod message_fixtures;
mod navigation_fixtures;
mod reader_fixtures;
mod sidebar_fixtures;

use compose_fixtures::{
    compose, draft, focus_on, open_reply, open_with, saved_reply, subject_only, type_in_draft,
    update_command,
};
use mira_assembly::api::DraftChanges;
use mira_tui::state::compose::Field;
use mira_tui::update::{ComposeIntent, update};
use reader_fixtures::offline_error;

/// La réponse d'un autre brouillon ne touche ni l'attente ni les champs du brouillon ouvert.
#[test]
fn saved_reply_for_another_draft_is_ignored() {
    let mut state = open_reply();
    focus_on(&mut state, Field::Subject);
    type_in_draft(&mut state, "!");
    compose(&mut state, ComposeIntent::Save);
    let other = draft("d9", &[], "Autre", "");
    update(&mut state, saved_reply("d9", Ok(other)));
    update(&mut state, saved_reply("d9", Err(offline_error())));
    let open = state.compose.open().unwrap();
    assert!(open.is_saving() && open.is_dirty(Field::Subject));
    assert!(state.status.message.is_none());
}

/// Une deuxième demande pendant l'enregistrement n'envoie rien de plus.
#[test]
fn save_while_saving_sends_nothing() {
    let mut state = open_reply();
    focus_on(&mut state, Field::Subject);
    type_in_draft(&mut state, "!");
    assert_eq!(compose(&mut state, ComposeIntent::Save).len(), 1);
    assert!(compose(&mut state, ComposeIntent::Save).is_empty());
}

/// Ce qui est tapé pendant l'enregistrement reste à enregistrer quand la réponse arrive.
#[test]
fn typing_during_save_stays_unsaved_after_the_reply() {
    let mut state = open_reply();
    focus_on(&mut state, Field::Subject);
    type_in_draft(&mut state, "!");
    compose(&mut state, ComposeIntent::Save);
    type_in_draft(&mut state, "?");
    let saved = draft("d1", &["bob@example.org"], "Re: Bonjour!", "Corps");
    update(&mut state, saved_reply("d1", Ok(saved)));
    let open = state.compose.open().unwrap();
    assert!(open.is_dirty(Field::Subject) && open.has_unsaved());
    let commands = compose(&mut state, ComposeIntent::Save);
    assert_eq!(
        commands,
        vec![update_command("d1", subject_only("Re: Bonjour!?"))]
    );
}

/// Une version HTML existante est effacée quand le texte change, pour ne pas envoyer un HTML périmé.
#[test]
fn editing_text_clears_existing_html_version() {
    let mut with_html = draft("d1", &["bob@example.org"], "Re: Bonjour", "Corps");
    with_html.body_html = Some("<p>Corps</p>".to_string());
    let mut state = open_with(with_html);
    focus_on(&mut state, Field::Body);
    type_in_draft(&mut state, "x");
    let commands = compose(&mut state, ComposeIntent::Save);
    let expected = DraftChanges {
        body_text: Some("xCorps".to_string()),
        body_html: Some(String::new()),
        ..DraftChanges::default()
    };
    assert_eq!(commands, vec![update_command("d1", expected)]);
}

/// Le corps rendu par le moteur avec des fins de ligne `\r\n` n'est pas pris pour une modification.
#[test]
fn crlf_body_from_the_engine_is_not_a_change() {
    let crlf = draft("d1", &["bob@example.org"], "Re: Bonjour", "a\r\nb");
    let mut state = open_with(crlf);
    assert!(!state.compose.open().unwrap().has_unsaved());
    assert!(compose(&mut state, ComposeIntent::Save).is_empty());
}
