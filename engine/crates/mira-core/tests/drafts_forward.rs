#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Brouillon de transfert : sans destinataire, objet `Fwd:` sans doublon, lien vers l'original.

mod support;
mod support_actions;
mod support_mirror;

use mira_core::drafts::{create_forward, draft_threading_headers, threading_headers};
use mira_vocabulary::DraftState;
use support_actions::mail::{incoming, store};
use support_actions::world::World;

/// Un transfert n'a aucun destinataire, un objet `Fwd:` et un lien vers le message d'origine
#[test]
fn forward_has_no_recipient_and_a_fwd_subject() {
    let world = World::seeded();
    let original = store(&world, &incoming("in-1", "Facture"));
    let draft = create_forward(&world.ctx(), &original, None).unwrap();
    assert!(draft.to.is_empty() && draft.cc.is_empty() && draft.bcc.is_empty());
    assert_eq!(draft.subject, "Fwd: Facture");
    assert_eq!(draft.in_reply_to, Some(original));
    assert_eq!(draft.state, DraftState::Editing);
    assert!(draft.attachments.is_empty());
}

/// `Fwd:` et `Fw:` déjà présents ne sont pas doublés ; `Re:` ne compte pas
#[test]
fn forward_subject_never_stacks_fwd_prefixes() {
    let world = World::seeded();
    let cases = [
        ("Fwd: Facture", "Fwd: Facture"),
        ("FW: Facture", "FW: Facture"),
        ("fwd : Facture", "fwd : Facture"),
        ("Re: Facture", "Fwd: Re: Facture"),
    ];
    for (index, (subject, expected)) in cases.into_iter().enumerate() {
        let original = store(&world, &incoming(&format!("in-{index}"), subject));
        let draft = create_forward(&world.ctx(), &original, None).unwrap();
        assert_eq!(draft.subject, expected);
    }
}

/// Les en-têtes de fil d'un transfert viennent aussi du message d'origine ; sans `Message-ID`, seules ses références restent
#[test]
fn forward_threading_headers_follow_the_original() {
    let world = World::seeded();
    let mut summary = incoming("in-1", "Facture");
    summary.message_id_header = None;
    let original = store(&world, &summary);
    let draft = create_forward(&world.ctx(), &original, None).unwrap();
    let headers = world
        .db
        .read(|tx| draft_threading_headers(tx, &draft))
        .unwrap();
    assert_eq!(headers.in_reply_to, None);
    assert_eq!(headers.references, ["<root@mail.example.org>"]);
    let plain = threading_headers(&incoming("in-2", "x"));
    assert_eq!(plain.references.len(), 2);
}

/// Un brouillon sans original, ou dont l'original a disparu du miroir, n'a pas d'en-têtes de fil
#[test]
fn draft_without_original_has_no_threading_headers() {
    let world = World::seeded();
    let original = store(&world, &incoming("in-1", "Facture"));
    let draft = create_forward(&world.ctx(), &original, None).unwrap();
    world.db.write(|tx| tx.delete_message(&original)).unwrap();
    let headers = world
        .db
        .read(|tx| draft_threading_headers(tx, &draft))
        .unwrap();
    assert!(headers.in_reply_to.is_none() && headers.references.is_empty());
}
