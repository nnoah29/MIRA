#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Échec TLS pendant un envoi : prouvé avant DATA, le message n'est pas parti, l'action reste à rejouer (AG-30).

mod support;
mod support_actions;
mod support_imap;
mod support_mirror;
mod support_replay;
mod support_smtp;

use mira_core::actions::{ReplayStop, SEND_UNDO_DELAY_MILLIS};
use mira_vocabulary::{ActionState, Security};
use support_imap::within;
use support_replay::presets::standard as started;
use support_replay::queue_view::state_of;
use support_replay::sends::schedule_ready_draft;

/// STARTTLS exigé mais absent : rien n'est parti, rien d'incertain, l'action attend un nouveau rejeu
#[tokio::test]
async fn tls_failure_before_data_keeps_the_send_pending_and_unsent() {
    within(async {
        let mut fx = started().await;
        fx.smtp_endpoint.security = Security::StartTls;
        let (_, action) = schedule_ready_draft(&fx, "k-tls");
        fx.advance(SEND_UNDO_DELAY_MILLIS);
        let mut client = fx.client().await;
        let report = fx.replay(&mut client).await;
        assert_eq!(report.uncertain, 0);
        let code = "tls.handshake".to_string();
        assert_eq!(report.stopped, Some(ReplayStop::Retry { code }));
        assert_eq!(state_of(&fx, &action), ActionState::Pending);
        assert!(fx.smtp.received().is_empty());
    })
    .await;
}
