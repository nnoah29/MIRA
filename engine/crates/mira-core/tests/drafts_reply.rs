#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Brouillons de réponse et de réponse à tous : destinataires, objet, lien vers l'original, en-têtes de fil.

mod support;
mod support_actions;
mod support_mirror;

use mira_core::drafts::{create_reply, create_reply_all, draft_threading_headers};
use mira_vocabulary::MessageId;
use support_actions::mail::{addresses, entry, incoming, store};
use support_actions::world::World;

/// Une réponse écrit à l'expéditeur, objet `Re:`, liée au message d'origine
#[test]
fn reply_goes_to_the_sender_with_a_re_subject() {
    let world = World::seeded();
    let original = store(&world, &incoming("in-1", "Réunion"));
    let draft = create_reply(&world.ctx(), &original, None).unwrap();
    assert_eq!(addresses(&draft.to), ["bob@example.org"]);
    assert!(draft.cc.is_empty() && draft.bcc.is_empty());
    assert_eq!(draft.subject, "Re: Réunion");
    assert_eq!(draft.in_reply_to, Some(original));
    assert_eq!(draft.identity.as_str(), "main");
}

/// Pas de doublon de préfixe : `Re:`, `RE:` et `Re :` sont reconnus, `Fwd:` ne l'est pas
#[test]
fn reply_subject_never_stacks_re_prefixes() {
    let world = World::seeded();
    let cases = [
        ("Re: Réunion", "Re: Réunion"),
        ("RE: Réunion", "RE: Réunion"),
        ("re : Réunion", "re : Réunion"),
        ("Fwd: Réunion", "Re: Fwd: Réunion"),
        (
            "Réunion sur le sujet Re: x",
            "Re: Réunion sur le sujet Re: x",
        ),
    ];
    for (index, (subject, expected)) in cases.into_iter().enumerate() {
        let original = store(&world, &incoming(&format!("in-{index}"), subject));
        let draft = create_reply(&world.ctx(), &original, None).unwrap();
        assert_eq!(draft.subject, expected);
    }
}

/// `Reply-To` l'emporte sur l'expéditeur ; répondre à son propre message écrit aux destinataires d'origine
#[test]
fn reply_prefers_reply_to_and_handles_own_messages() {
    let world = World::seeded();
    let mut with_reply_to = incoming("in-1", "Sujet");
    with_reply_to.reply_to = vec![entry("list@example.org")];
    let original = store(&world, &with_reply_to);
    let draft = create_reply(&world.ctx(), &original, None).unwrap();
    assert_eq!(addresses(&draft.to), ["list@example.org"]);
    let mut own = incoming("in-2", "Sujet");
    own.from = vec![entry("Main@Example.org")];
    let original = store(&world, &own);
    let draft = create_reply(&world.ctx(), &original, None).unwrap();
    assert_eq!(
        addresses(&draft.to),
        ["main@example.org", "carol@example.org"]
    );
}

/// Répondre à tous : l'expéditeur en destinataire, les autres en copie, sans soi-même ni doublon
#[test]
fn reply_all_copies_the_other_recipients_without_self_or_duplicates() {
    let world = World::seeded();
    let mut summary = incoming("in-1", "Sujet");
    summary.to.push(entry("CAROL@example.org"));
    summary.cc.push(entry("alice@example.org"));
    summary.cc.push(entry("bob@example.org"));
    let original = store(&world, &summary);
    let draft = create_reply_all(&world.ctx(), &original, None).unwrap();
    assert_eq!(addresses(&draft.to), ["bob@example.org"]);
    assert_eq!(
        addresses(&draft.cc),
        ["carol@example.org", "dave@example.org"]
    );
    assert_eq!(draft.subject, "Re: Sujet");
}

/// `In-Reply-To` est le `Message-ID` d'origine ; `References` prolonge celles de l'original
#[test]
fn threading_headers_come_from_the_original_message() {
    let world = World::seeded();
    let original = store(&world, &incoming("in-1", "Sujet"));
    let draft = create_reply(&world.ctx(), &original, None).unwrap();
    let headers = world
        .db
        .read(|tx| draft_threading_headers(tx, &draft))
        .unwrap();
    assert_eq!(
        headers.in_reply_to.as_deref(),
        Some("<in-1@mail.example.org>")
    );
    assert_eq!(
        headers.references,
        ["<root@mail.example.org>", "<in-1@mail.example.org>"]
    );
}

/// Répondre à un message inconnu donne `not_found.message`
#[test]
fn reply_to_unknown_message_is_refused() {
    let world = World::seeded();
    let error = create_reply(&world.ctx(), &MessageId::from_raw("nope"), None).unwrap_err();
    assert_eq!(error.code, "not_found.message");
}
