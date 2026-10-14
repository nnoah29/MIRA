#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Plantage pendant le rejeu : « en cours » écrit avant le serveur (inv. 17), un seul effet après reprise (inv. 18).

mod support;
mod support_actions;
mod support_imap;
mod support_mirror;
mod support_replay;
mod support_smtp;

use std::time::Duration;

use mira_core::actions::{UNDO_DELAY_MILLIS, recover_interrupted_actions};
use mira_core::imap::ImapClient;
use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::{ActionId, ActionKind, ActionState, MailboxId, ProviderQuirks, Security};
use support_actions::inspect::force_state;
use support_actions::requests::submit_folders;
use support_imap::env::local_endpoint;
use support_imap::logins::password_login;
use support_imap::mail::mail;
use support_imap::scripted::scripted;
use support_imap::within;
use support_replay::fixture::Fixture;
use support_replay::observe::{server_flags, server_message_ids};
use support_replay::queue_view::state_of;

/// Banc standard avec « déplacer msg-1 vers Projects » prêt à partir
async fn move_ready() -> (Fixture, ActionId) {
    let fx = Fixture::start(CapabilitySet::full(), ProviderQuirks::default()).await;
    let to = MailboxId::from_raw("box-proj");
    let id = submit_folders(&fx.world, ActionKind::Move { to }, &["msg-1"], "k-crash");
    fx.advance(UNDO_DELAY_MILLIS + 1);
    (fx, id)
}

/// Inv. 17 : le serveur se tait en pleine exécution et le processus « meurt » ; l'état « en cours » était déjà écrit
#[tokio::test]
async fn invariant_17_action_written_before_attempt() {
    within(async {
        let fx = Fixture::start(CapabilitySet::full(), ProviderQuirks::default()).await;
        let read = ActionKind::MarkRead { read: true };
        let id = submit_folders(&fx.world, read, &["msg-1"], "k-17");
        fx.advance(UNDO_DELAY_MILLIS + 1);
        assert_eq!(state_of(&fx, &id), ActionState::Pending);
        let greeting = "* OK [CAPABILITY IMAP4rev1] prêt\r\n";
        let logged_in = "{tag} OK [CAPABILITY IMAP4rev1 UIDPLUS] connecté\r\n";
        let (port, task) = scripted(greeting, &[logged_in]).await;
        let target = local_endpoint(port, Security::Plaintext);
        let mut client = ImapClient::connect(&target, &password_login("secret"))
            .await
            .unwrap();
        let crashed = tokio::time::timeout(Duration::from_millis(300), fx.replay(&mut client));
        assert!(
            crashed.await.is_err(),
            "le serveur muet bloque la tentative"
        );
        assert_eq!(state_of(&fx, &id), ActionState::InProgress);
        task.abort();
    })
    .await;
}

/// Inv. 18 : le serveur a déjà déplacé le message, le plantage a effacé la trace ; le rejeu n'ajoute aucun second effet
#[tokio::test]
async fn invariant_18_replay_after_crash_has_single_effect() {
    within(async {
        let (fx, id) = move_ready().await;
        let mut client = fx.client().await;
        fx.replay(&mut client).await;
        force_state(&fx.world, &id, ActionState::InProgress);
        let recovered = recover_interrupted_actions(&fx.ctx(), &mut client).await;
        assert_eq!(recovered.unwrap().requeued, 1);
        assert_eq!(fx.replay(&mut client).await.succeeded, 1);
        assert_eq!(server_message_ids(&fx, "Projects"), ["msg-1@example.org"]);
        assert_eq!(
            server_message_ids(&fx, "INBOX"),
            ["msg-2@example.org", "msg-3@example.org"]
        );
        assert_eq!(state_of(&fx, &id), ActionState::Succeeded);
    })
    .await;
}

/// Inv. 18 : la copie est faite mais la source n'est pas purgée ; le rejeu ne copie pas une seconde fois
#[tokio::test]
async fn replay_after_half_done_copy_does_not_duplicate_the_message() {
    within(async {
        let (fx, id) = move_ready().await;
        let copy = mail("Sujet msg-1", "msg-1@example.org");
        fx.env.store.deliver("alice", "Projects", &copy, &[]);
        force_state(&fx.world, &id, ActionState::InProgress);
        let mut client = fx.client().await;
        recover_interrupted_actions(&fx.ctx(), &mut client)
            .await
            .unwrap();
        assert_eq!(fx.replay(&mut client).await.succeeded, 1);
        assert_eq!(server_message_ids(&fx, "Projects"), ["msg-1@example.org"]);
        assert!(!server_message_ids(&fx, "INBOX").contains(&"msg-1@example.org".to_string()));
        assert!(server_flags(&fx, "Projects", "msg-1@example.org").is_empty());
    })
    .await;
}

/// Une action interrompue est remise en file par la reprise ; une action en attente n'est pas touchée
#[tokio::test]
async fn recovery_requeues_only_interrupted_actions() {
    within(async {
        let (fx, id) = move_ready().await;
        let read = ActionKind::MarkRead { read: true };
        let other = submit_folders(&fx.world, read, &["msg-2"], "k-other");
        force_state(&fx.world, &id, ActionState::InProgress);
        let mut client = fx.client().await;
        let report = recover_interrupted_actions(&fx.ctx(), &mut client).await;
        assert_eq!(report.unwrap().requeued, 1);
        assert_eq!(state_of(&fx, &id), ActionState::Pending);
        assert_eq!(state_of(&fx, &other), ActionState::Pending);
    })
    .await;
}
