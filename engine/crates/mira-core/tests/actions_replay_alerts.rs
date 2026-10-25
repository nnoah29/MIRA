#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Alertes serveur au rejeu : boîte pleine et débit limité sont annoncés en plus de l'échec, un échec ordinaire non.

mod support;
mod support_actions;
mod support_imap;
mod support_mirror;
mod support_replay;
mod support_smtp;

use mira_core::actions::{SEND_UNDO_DELAY_MILLIS, UNDO_DELAY_MILLIS};
use mira_vocabulary::{ActionKind, EventKind, MailboxId};
use support_actions::requests::submit_folders;
use support_imap::within;
use support_replay::presets::standard as started;
use support_replay::sends::schedule_ready_draft;

/// Événements d'alerte émis jusqu'ici : (« quota » ou « débit », compte)
fn alerts(fx: &support_replay::fixture::Fixture) -> Vec<(&'static str, String)> {
    let named = |kind: EventKind| match kind {
        EventKind::QuotaFull { account } => Some(("quota", account.to_string())),
        EventKind::RateLimited { account } => Some(("débit", account.to_string())),
        _ => None,
    };
    fx.world
        .sink
        .kinds()
        .into_iter()
        .filter_map(named)
        .collect()
}

/// Un refus de quota au déplacement annonce « boîte pleine » pour le compte, en plus de l'échec de l'action
#[tokio::test]
async fn quota_refusal_announces_a_full_mailbox() {
    within(async {
        let fx = started().await;
        let to = MailboxId::from_raw("box-proj");
        submit_folders(&fx.world, ActionKind::Move { to }, &["msg-1"], "k-quota");
        fx.env
            .store
            .fail_next("UID MOVE", "NO [OVERQUOTA] mailbox full");
        fx.advance(UNDO_DELAY_MILLIS + 1);
        let mut client = fx.client().await;
        fx.replay(&mut client).await;
        let found = alerts(&fx);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].0, "quota");
        assert_eq!(found[0].1, "acc-1");
    })
    .await;
}

/// Un envoi mis en liste grise (451) annonce « débit limité » et reste en attente
#[tokio::test]
async fn greylisted_send_announces_rate_limiting() {
    within(async {
        let fx = started().await;
        schedule_ready_draft(&fx, "k-grey");
        fx.smtp.fail_next("451 4.7.1 Greylisted, try later");
        fx.advance(SEND_UNDO_DELAY_MILLIS);
        let mut client = fx.client().await;
        fx.replay(&mut client).await;
        let found = alerts(&fx);
        assert_eq!(found.iter().map(|a| a.0).collect::<Vec<_>>(), ["débit"]);
    })
    .await;
}

/// Un échec ordinaire (message disparu) n'annonce aucune alerte
#[tokio::test]
async fn ordinary_failure_announces_no_alert() {
    within(async {
        let fx = started().await;
        let read = ActionKind::MarkRead { read: true };
        submit_folders(&fx.world, read, &["msg-1"], "k-plain");
        fx.env.store.expunge("alice", "INBOX", 1);
        fx.advance(UNDO_DELAY_MILLIS + 1);
        let mut client = fx.client().await;
        fx.replay(&mut client).await;
        assert!(alerts(&fx).is_empty());
    })
    .await;
}
