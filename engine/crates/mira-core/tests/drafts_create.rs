#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Création d'un brouillon neuf : identité par défaut ou choisie, compte, persistance.

mod support;
mod support_actions;
mod support_mirror;

use mira_core::drafts::create_blank;
use mira_vocabulary::{AccountId, DraftState, IdentityId};
use support::accounts::sample_identity;
use support_actions::world::{START_MILLIS, World};

/// Un brouillon neuf est vide, en édition, sous l'identité par défaut, et relisible en base
#[test]
fn blank_draft_uses_the_default_identity_and_is_stored() {
    let world = World::seeded();
    let draft = create_blank(&world.ctx(), &AccountId::from_raw("acc-1"), None).unwrap();
    assert_eq!(draft.identity.as_str(), "main");
    assert_eq!(draft.state, DraftState::Editing);
    assert!(draft.to.is_empty() && draft.subject.is_empty() && draft.body_text.is_empty());
    assert_eq!(draft.in_reply_to, None);
    assert_eq!(draft.updated_at.as_millis(), START_MILLIS);
    let stored = world.db.read(|tx| tx.get_draft(&draft.id)).unwrap();
    assert_eq!(stored, Some(draft));
}

/// L'identité demandée est utilisée si elle appartient au compte
#[test]
fn blank_draft_uses_the_requested_identity() {
    let world = World::seeded();
    world
        .db
        .write(|tx| tx.insert_identity(&sample_identity("alias", "acc-1", false)))
        .unwrap();
    let wanted = IdentityId::from_raw("alias");
    let account = AccountId::from_raw("acc-1");
    let draft = create_blank(&world.ctx(), &account, Some(&wanted)).unwrap();
    assert_eq!(draft.identity, wanted);
}

/// Une identité inconnue ou d'un autre compte est refusée (`not_found.identity`)
#[test]
fn unknown_or_foreign_identity_is_refused() {
    let world = World::seeded();
    world
        .db
        .write(|tx| tx.insert_identity(&sample_identity("other", "acc-2", true)))
        .unwrap();
    let account = AccountId::from_raw("acc-1");
    for id in ["other", "nope"] {
        let wanted = IdentityId::from_raw(id);
        let error = create_blank(&world.ctx(), &account, Some(&wanted)).unwrap_err();
        assert_eq!(error.code, "not_found.identity");
    }
}

/// Un compte inconnu, ou sans aucune identité, ne permet pas de créer de brouillon
#[test]
fn account_without_identity_or_unknown_account_is_refused() {
    let world = World::seeded();
    let no_identity = create_blank(&world.ctx(), &AccountId::from_raw("acc-2"), None).unwrap_err();
    assert_eq!(no_identity.code, "not_found.identity");
    let unknown = create_blank(&world.ctx(), &AccountId::from_raw("acc-nope"), None).unwrap_err();
    assert_eq!(unknown.code, "not_found.account");
    let listed = world
        .db
        .read(|tx| tx.list_drafts(&AccountId::from_raw("acc-2")))
        .unwrap();
    assert!(listed.is_empty());
}
