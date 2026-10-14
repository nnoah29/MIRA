#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Rejeu des déplacements et des drapeaux contre le faux serveur IMAP : S7, inv. 19, D-14, repli sans MOVE ni UIDPLUS.

mod support;
mod support_actions;
mod support_imap;
mod support_mirror;
mod support_replay;
mod support_smtp;

use mira_core::actions::UNDO_DELAY_MILLIS;
use mira_testkit::imap::{Capability, CapabilitySet};
use mira_vocabulary::{ActionKind, MailboxId, ProviderQuirks};
use support_actions::inspect::boxes_of;
use support_actions::requests::submit_folders;
use support_imap::within;
use support_replay::fixture::Fixture;
use support_replay::observe::{command_count, command_position, server_flags, server_message_ids};

/// Soumet « déplacer msg-1 vers Projects » puis « marquer msg-1 lu », sans aucun accès au serveur
async fn queue_move_then_read(fx: &Fixture) {
    let to = MailboxId::from_raw("box-proj");
    submit_folders(&fx.world, ActionKind::Move { to }, &["msg-1"], "k-move");
    submit_folders(
        &fx.world,
        ActionKind::MarkRead { read: true },
        &["msg-1"],
        "k-read",
    );
    assert_eq!(boxes_of(&fx.world.db, "msg-1"), vec!["box-proj"]);
    assert_eq!(fx.env.server.connection_count(), 0);
    fx.advance(UNDO_DELAY_MILLIS + 1);
}

/// S7 avec MOVE : déplacement hors ligne, puis rejeu dans l'ordre ; le drapeau suit le message déplacé
#[tokio::test]
async fn scenario_s7_move_offline_then_replay_in_order() {
    within(async {
        let fx = Fixture::start(CapabilitySet::full(), ProviderQuirks::default()).await;
        queue_move_then_read(&fx).await;
        let mut client = fx.client().await;
        let report = fx.replay(&mut client).await;
        assert_eq!((report.succeeded, report.failed), (2, 0));
        assert_eq!(
            server_message_ids(&fx, "INBOX"),
            ["msg-2@example.org", "msg-3@example.org"]
        );
        assert_eq!(server_message_ids(&fx, "Projects"), ["msg-1@example.org"]);
        assert_eq!(
            server_flags(&fx, "Projects", "msg-1@example.org"),
            ["\\Seen"]
        );
        let moved = command_position(&fx, "UID MOVE").unwrap();
        let flagged = command_position(&fx, "+FLAGS").unwrap();
        assert!(moved < flagged, "le déplacement précède le drapeau");
    })
    .await;
}

/// S7 sans MOVE : COPY, puis `\Deleted` et UID EXPUNGE, puis le drapeau
#[tokio::test]
async fn scenario_s7_move_without_move_extension_copies_then_expunges() {
    within(async {
        let fx = Fixture::start(CapabilitySet::without_move(), ProviderQuirks::default()).await;
        queue_move_then_read(&fx).await;
        let mut client = fx.client().await;
        let report = fx.replay(&mut client).await;
        assert_eq!(report.succeeded, 2);
        assert_eq!(command_count(&fx, "UID MOVE"), 0);
        assert!(
            command_position(&fx, "UID COPY").unwrap()
                < command_position(&fx, "UID EXPUNGE").unwrap()
        );
        assert_eq!(
            server_message_ids(&fx, "INBOX"),
            ["msg-2@example.org", "msg-3@example.org"]
        );
        assert_eq!(server_message_ids(&fx, "Projects"), ["msg-1@example.org"]);
        assert_eq!(
            server_flags(&fx, "Projects", "msg-1@example.org"),
            ["\\Seen"]
        );
    })
    .await;
}

/// Sans MOVE ni UIDPLUS : copie faite, source seulement marquée `\Deleted`, aucune purge (10.14)
#[tokio::test]
async fn move_without_uidplus_marks_deleted_and_never_purges() {
    within(async {
        let caps = CapabilitySet::without_move().without(Capability::UidPlus);
        let fx = Fixture::start(caps, ProviderQuirks::default()).await;
        queue_move_then_read(&fx).await;
        let mut client = fx.client().await;
        fx.replay(&mut client).await;
        assert_eq!(command_count(&fx, "EXPUNGE"), 0);
        assert_eq!(
            server_flags(&fx, "INBOX", "msg-1@example.org"),
            ["\\Deleted"]
        );
        assert_eq!(server_message_ids(&fx, "Projects"), ["msg-1@example.org"]);
    })
    .await;
}

/// D-14 : « lu » est posé sur toutes les apparitions du message (réception et « Work »)
#[tokio::test]
async fn mark_read_reaches_every_appearance_of_the_message() {
    within(async {
        let fx = Fixture::start(CapabilitySet::full(), ProviderQuirks::default()).await;
        let read = ActionKind::MarkRead { read: true };
        submit_folders(&fx.world, read, &["msg-3"], "k-read-3");
        fx.advance(UNDO_DELAY_MILLIS + 1);
        let mut client = fx.client().await;
        fx.replay(&mut client).await;
        let seen = ["\\Seen"];
        assert_eq!(server_flags(&fx, "INBOX", "msg-3@example.org"), seen);
        assert_eq!(server_flags(&fx, "Work", "msg-3@example.org"), seen);
        assert!(server_flags(&fx, "INBOX", "msg-1@example.org").is_empty());
    })
    .await;
}

/// Inv. 19 : les actions sont jouées dans l'ordre de soumission, pas dans l'ordre des messages
#[tokio::test]
async fn invariant_19_actions_replayed_in_submission_order() {
    within(async {
        let fx = Fixture::start(CapabilitySet::full(), ProviderQuirks::default()).await;
        let read = || ActionKind::MarkRead { read: true };
        submit_folders(&fx.world, read(), &["msg-2"], "k-a");
        submit_folders(&fx.world, read(), &["msg-1"], "k-b");
        let flagged = ActionKind::MarkFlagged { flagged: true };
        submit_folders(&fx.world, flagged, &["msg-2"], "k-c");
        fx.advance(UNDO_DELAY_MILLIS + 1);
        let mut client = fx.client().await;
        let report = fx.replay(&mut client).await;
        assert_eq!(report.succeeded, 3);
        let steps = [
            "STORE 2 +FLAGS.SILENT (\\Seen)",
            "STORE 1 +FLAGS.SILENT (\\Seen)",
            "STORE 2 +FLAGS.SILENT (\\Flagged)",
        ];
        let positions: Vec<usize> = steps
            .iter()
            .map(|needle| command_position(&fx, needle).unwrap())
            .collect();
        assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    })
    .await;
}
