#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Copie serveur du brouillon : supprimée après l'envoi et à la suppression du brouillon.

mod support;
mod support_actions;
mod support_imap;
mod support_mirror;
mod support_replay;
mod support_smtp;

use mira_core::actions::{SEND_UNDO_DELAY_MILLIS, UNDO_DELAY_MILLIS};
use mira_core::drafts::delete_draft;
use mira_core::imap::ImapClient;
use mira_vocabulary::{ActionKind, Draft};
use support_actions::drafts::ready_draft;
use support_actions::requests::submit_folders;
use support_imap::within;
use support_replay::fixture::Fixture;
use support_replay::observe::server_message_ids;
use support_replay::presets::standard as started;
use support_replay::sends::schedule_ready_draft;

/// Dépose la copie serveur du brouillon puis rejoue la file
async fn save_copy(fx: &Fixture, client: &mut ImapClient, draft: &Draft) {
    let save = ActionKind::SaveDraft {
        draft: draft.id.clone(),
    };
    submit_folders(&fx.world, save, &[], "k-save");
    fx.advance(UNDO_DELAY_MILLIS + 1);
    assert_eq!(fx.replay(client).await.succeeded, 1);
    assert_eq!(server_message_ids(fx, "Drafts").len(), 1);
}

/// Après l'envoi, la copie du brouillon disparaît de la boîte Brouillons du serveur
#[tokio::test]
async fn regression_send_removes_the_server_copy_of_the_draft() {
    within(async {
        let fx = started().await;
        let mut client = fx.client().await;
        let (draft, _) = schedule_ready_draft(&fx, "k-send");
        save_copy(&fx, &mut client, &draft).await;
        fx.advance(SEND_UNDO_DELAY_MILLIS);
        assert_eq!(fx.replay(&mut client).await.succeeded, 1);
        assert!(server_message_ids(&fx, "Drafts").is_empty());
        assert_eq!(server_message_ids(&fx, "Sent").len(), 1);
    })
    .await;
}

/// Supprimer un brouillon déjà copié sur le serveur supprime aussi la copie, au prochain rejeu
#[tokio::test]
async fn regression_deleting_a_saved_draft_removes_its_server_copy() {
    within(async {
        let fx = started().await;
        let mut client = fx.client().await;
        let draft = ready_draft(&fx.world);
        save_copy(&fx, &mut client, &draft).await;
        delete_draft(&fx.world.ctx(), &draft.id).unwrap();
        fx.advance(UNDO_DELAY_MILLIS + 1);
        assert_eq!(fx.replay(&mut client).await.succeeded, 1);
        assert!(server_message_ids(&fx, "Drafts").is_empty());
    })
    .await;
}

/// Supprimer un brouillon jamais copié sur le serveur ne met rien en file
#[tokio::test]
async fn deleting_a_draft_never_saved_queues_nothing() {
    within(async {
        let fx = started().await;
        let draft = ready_draft(&fx.world);
        delete_draft(&fx.world.ctx(), &draft.id).unwrap();
        let queued = fx
            .world
            .db
            .read(|tx| tx.table_row_count("actions"))
            .unwrap();
        assert_eq!(queued, 0);
    })
    .await;
}
