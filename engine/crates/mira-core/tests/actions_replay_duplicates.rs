#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Déplacement face à des doublons de Message-ID : jamais de purge de la source sur une simple coïncidence (inv. 18).

mod support;
mod support_actions;
mod support_imap;
mod support_mirror;
mod support_replay;
mod support_smtp;

use mira_core::actions::UNDO_DELAY_MILLIS;
use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::{ActionKind, MailboxId, ProviderQuirks};
use support_actions::requests::submit_folders;
use support_imap::mail::mail;
use support_imap::within;
use support_replay::fixture::Fixture;
use support_replay::observe::server_message_ids;

/// Banc standard avec « déplacer msg-1 vers Projects » soumis et prêt à partir
async fn move_ready() -> Fixture {
    let fx = Fixture::start(CapabilitySet::full(), ProviderQuirks::default()).await;
    let to = MailboxId::from_raw("box-proj");
    submit_folders(&fx.world, ActionKind::Move { to }, &["msg-1"], "k-dup");
    fx.advance(UNDO_DELAY_MILLIS + 1);
    fx
}

/// Nombre d'exemplaires de msg-1 dans la boîte du serveur
fn copies_of_first(fx: &Fixture, mailbox: &str) -> usize {
    let ids = server_message_ids(fx, mailbox);
    ids.iter().filter(|id| *id == "msg-1@example.org").count()
}

/// Un autre message de même Message-ID déjà présent dans la destination ne fait pas purger la source
#[tokio::test]
async fn regression_same_message_id_with_other_content_does_not_purge_the_source() {
    within(async {
        let fx = move_ready().await;
        let other = mail("Un tout autre sujet, bien plus long", "msg-1@example.org");
        fx.env.store.deliver("alice", "Projects", &other, &[]);
        let mut client = fx.client().await;
        let report = fx.replay(&mut client).await;
        assert_eq!((report.succeeded, report.failed), (1, 0));
        assert_eq!(copies_of_first(&fx, "Projects"), 2);
        assert_eq!(copies_of_first(&fx, "INBOX"), 0);
    })
    .await;
}

/// Un UID périmé et plusieurs messages de même Message-ID : ambiguïté, rien n'est déplacé ni purgé
#[tokio::test]
async fn regression_ambiguous_message_id_moves_nothing() {
    within(async {
        let fx = move_ready().await;
        let twin = mail("Sujet msg-1", "msg-1@example.org");
        fx.env.store.deliver("alice", "INBOX", &twin, &[]);
        fx.env.store.renumber("alice", "INBOX");
        let mut client = fx.client().await;
        let report = fx.replay(&mut client).await;
        assert_eq!((report.succeeded, report.failed), (0, 1));
        assert_eq!(copies_of_first(&fx, "INBOX"), 2);
        assert_eq!(copies_of_first(&fx, "Projects"), 0);
    })
    .await;
}

/// Une copie identique déjà arrivée (reprise après plantage) est reconnue : la source est purgée sans seconde copie
#[tokio::test]
async fn identical_copy_in_destination_is_recognized_as_already_moved() {
    within(async {
        let fx = move_ready().await;
        let copy = mail("Sujet msg-1", "msg-1@example.org");
        fx.env.store.deliver("alice", "Projects", &copy, &[]);
        let mut client = fx.client().await;
        let report = fx.replay(&mut client).await;
        assert_eq!(report.succeeded, 1);
        assert_eq!(copies_of_first(&fx, "Projects"), 1);
        assert_eq!(copies_of_first(&fx, "INBOX"), 0);
    })
    .await;
}
