#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Libellés système à la synchronisation : « en attente » après le délai, « mentions », levée par une réponse.

mod support_labels;
mod support_sync;

use mira_core::system_labels::set_system_label;
use mira_vocabulary::{EventKind, SystemLabelKind as Kind};
use support_labels::dated::{bob_reply, from_bob, my_reply};
use support_labels::world::{kinds_of, open, pass, thread_by_subject};
use support_sync::waiting::count_events;
use support_sync::within;

/// Ma réponse vieille de trois jours : le fil est « en attente » et la synchronisation l'annonce
#[tokio::test]
async fn my_old_reply_makes_the_thread_waiting() {
    within(async {
        let (world, mut session) = open().await;
        world.store.deliver(
            "alice",
            "INBOX",
            &from_bob("Devis", "d1@x", 4, "Un devis ?"),
            &[],
        );
        world
            .store
            .deliver("alice", "Sent", &my_reply("Devis", "d2@x", "d1@x", 3), &[]);
        pass(&mut session).await;
        assert_eq!(kinds_of(&world, "Devis"), vec![Kind::Waiting]);
        let changed = |k: &EventKind| matches!(k, EventKind::SystemLabelsChanged);
        assert_eq!(count_events(&world.recording, changed), 1);
    })
    .await;
}

/// Ma réponse de la veille : pas encore « en attente »
#[tokio::test]
async fn a_recent_reply_is_not_waiting_yet() {
    within(async {
        let (world, mut session) = open().await;
        world.store.deliver(
            "alice",
            "INBOX",
            &from_bob("Devis", "d1@x", 2, "Un devis ?"),
            &[],
        );
        world
            .store
            .deliver("alice", "Sent", &my_reply("Devis", "d2@x", "d1@x", 1), &[]);
        pass(&mut session).await;
        assert!(kinds_of(&world, "Devis").is_empty());
    })
    .await;
}

/// Une réponse de Bob arrivée ensuite lève l'attente
#[tokio::test]
async fn an_answer_lifts_the_waiting() {
    within(async {
        let (world, mut session) = open().await;
        world.store.deliver(
            "alice",
            "INBOX",
            &from_bob("Devis", "d1@x", 4, "Un devis ?"),
            &[],
        );
        world
            .store
            .deliver("alice", "Sent", &my_reply("Devis", "d2@x", "d1@x", 3), &[]);
        pass(&mut session).await;
        let answer = bob_reply("Devis", "d3@x", "d2@x", 1, "Merci");
        world.store.deliver("alice", "INBOX", &answer, &[]);
        pass(&mut session).await;
        assert!(kinds_of(&world, "Devis").is_empty());
    })
    .await;
}

/// Mon message qui n'est que dans la corbeille n'attend rien
#[tokio::test]
async fn a_reply_in_the_trash_does_not_count() {
    within(async {
        let (world, mut session) = open().await;
        world.store.deliver(
            "alice",
            "INBOX",
            &from_bob("Devis", "d1@x", 4, "Un devis ?"),
            &[],
        );
        world
            .store
            .deliver("alice", "Trash", &my_reply("Devis", "d2@x", "d1@x", 3), &[]);
        pass(&mut session).await;
        assert!(kinds_of(&world, "Devis").is_empty());
    })
    .await;
}

/// `@alice` dans le dernier message reçu : mention ; une salutation ou un fil de plus de 30 jours : non
#[tokio::test]
async fn mentions_are_explicit_and_recent() {
    within(async {
        let (world, mut session) = open().await;
        world.store.deliver(
            "alice",
            "INBOX",
            &from_bob("Avis", "m1@x", 1, "Merci @Alice de relire"),
            &[],
        );
        world.store.deliver(
            "alice",
            "INBOX",
            &from_bob("Salut", "m2@x", 1, "Bonjour Alice"),
            &[],
        );
        world.store.deliver(
            "alice",
            "INBOX",
            &from_bob("Ancien", "m3@x", 40, "Merci @Alice"),
            &[],
        );
        pass(&mut session).await;
        assert_eq!(kinds_of(&world, "Avis"), vec![Kind::Mention]);
        assert!(kinds_of(&world, "Salut").is_empty());
        assert!(kinds_of(&world, "Ancien").is_empty());
    })
    .await;
}

/// Un retrait manuel tient aux passes suivantes, mais pas face à un message plus récent
#[tokio::test]
async fn a_manual_removal_holds_until_a_newer_message() {
    within(async {
        let (world, mut session) = open().await;
        world.store.deliver(
            "alice",
            "INBOX",
            &from_bob("Avis", "m1@x", 2, "Merci @Alice"),
            &[],
        );
        pass(&mut session).await;
        let thread = thread_by_subject(&world, "Avis");
        world
            .db
            .write(|tx| set_system_label(tx, Kind::Mention, &[thread], false))
            .unwrap();
        pass(&mut session).await;
        assert!(kinds_of(&world, "Avis").is_empty());
        let newer = bob_reply("Avis", "m4@x", "m1@x", 1, "@Alice et ensuite ?");
        world.store.deliver("alice", "INBOX", &newer, &[]);
        pass(&mut session).await;
        assert_eq!(kinds_of(&world, "Avis"), vec![Kind::Mention]);
    })
    .await;
}
