#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Modification et suppression d'un brouillon.

mod support;
mod support_actions;
mod support_mirror;

use mira_core::drafts::{DraftEdit, delete_draft, edit_draft};
use mira_vocabulary::{DraftId, DraftState, Timestamp};
use support_actions::drafts::{blank, stored};
use support_actions::mail::entry;
use support_actions::world::World;

/// Les champs donnés changent, les autres restent ; la date de modification suit l'horloge
#[test]
fn edit_changes_only_the_given_fields() {
    let world = World::seeded();
    let draft = blank(&world);
    world.clock.advance_millis(7_000);
    let edit = DraftEdit {
        to: Some(vec![entry("bob@example.org")]),
        subject: Some("Bonjour".to_string()),
        body_text: Some("Texte".to_string()),
        body_html: Some(Some("<p>Texte</p>".to_string())),
        ..DraftEdit::default()
    };
    let edited = edit_draft(&world.ctx(), &draft.id, edit).unwrap();
    assert_eq!(edited.to, vec![entry("bob@example.org")]);
    assert_eq!(edited.subject, "Bonjour");
    assert_eq!(edited.body_html.as_deref(), Some("<p>Texte</p>"));
    assert_eq!(edited.identity, draft.identity);
    assert_eq!(
        edited.updated_at,
        Timestamp::from_millis(draft.updated_at.as_millis() + 7_000)
    );
    assert_eq!(stored(&world, &draft), edited);
}

/// `body_html: Some(None)` efface la version HTML sans toucher au reste
#[test]
fn edit_can_clear_the_html_body_and_keep_other_fields() {
    let world = World::seeded();
    let draft = blank(&world);
    let first = DraftEdit {
        subject: Some("Bonjour".to_string()),
        body_html: Some(Some("<p>x</p>".to_string())),
        ..DraftEdit::default()
    };
    edit_draft(&world.ctx(), &draft.id, first).unwrap();
    let clear = DraftEdit {
        body_html: Some(None),
        cc: Some(vec![entry("carol@example.org")]),
        ..DraftEdit::default()
    };
    let cleared = edit_draft(&world.ctx(), &draft.id, clear).unwrap();
    assert_eq!(cleared.body_html, None);
    assert_eq!(cleared.subject, "Bonjour");
    assert_eq!(cleared.cc, vec![entry("carol@example.org")]);
}

/// Une adresse mal formée est refusée et le brouillon reste intact
#[test]
fn edit_with_an_invalid_address_is_refused() {
    let world = World::seeded();
    let draft = blank(&world);
    let edit = DraftEdit {
        bcc: Some(vec![entry("pas-une-adresse")]),
        subject: Some("Perdu".to_string()),
        ..DraftEdit::default()
    };
    let error = edit_draft(&world.ctx(), &draft.id, edit).unwrap_err();
    assert_eq!(error.code, "invalid_input.address");
    let stored = world
        .db
        .read(|tx| tx.get_draft(&draft.id))
        .unwrap()
        .unwrap();
    assert_eq!(stored.subject, "");
    assert!(stored.bcc.is_empty());
}

/// Un brouillon planifié ou en envoi n'est plus modifiable ; un brouillon inconnu non plus
#[test]
fn edit_refuses_scheduled_sending_and_unknown_drafts() {
    let world = World::seeded();
    let draft = blank(&world);
    for state in [
        DraftState::Scheduled {
            at: Timestamp::from_millis(1),
        },
        DraftState::Sending,
    ] {
        let mut changed = draft.clone();
        changed.state = state;
        world.db.write(|tx| tx.update_draft(&changed)).unwrap();
        let error = edit_draft(&world.ctx(), &draft.id, DraftEdit::default()).unwrap_err();
        assert_eq!(error.code, "incompatible_state.draft_not_editable");
    }
    let error = edit_draft(
        &world.ctx(),
        &DraftId::from_raw("nope"),
        DraftEdit::default(),
    );
    assert_eq!(error.unwrap_err().code, "not_found.draft");
}

/// Supprimer retire le brouillon ; un brouillon planifié ou inconnu est refusé
#[test]
fn delete_removes_editing_drafts_only() {
    let world = World::seeded();
    let draft = blank(&world);
    delete_draft(&world.ctx(), &draft.id).unwrap();
    assert_eq!(world.db.read(|tx| tx.get_draft(&draft.id)).unwrap(), None);
    let error = delete_draft(&world.ctx(), &draft.id).unwrap_err();
    assert_eq!(error.code, "not_found.draft");
    let scheduled = blank(&world);
    let mut changed = scheduled.clone();
    changed.state = DraftState::Scheduled {
        at: Timestamp::from_millis(1),
    };
    world.db.write(|tx| tx.update_draft(&changed)).unwrap();
    let error = delete_draft(&world.ctx(), &scheduled.id).unwrap_err();
    assert_eq!(error.code, "incompatible_state.draft_not_editable");
}
