#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Échec du rafraîchissement du jeton pendant un envoi : l'envoi n'est pas parti, il n'est donc pas incertain.

mod support;
mod support_actions;
mod support_imap;
mod support_mirror;
mod support_replay;
mod support_smtp;

use mira_core::actions::{ReplayStop, SEND_UNDO_DELAY_MILLIS};
use mira_vocabulary::ActionState;
use support_imap::within;
use support_replay::credentials::FakeCredentials;
use support_replay::presets::standard as started;
use support_replay::queue_view::state_of;
use support_replay::sends::schedule_ready_draft;

/// Un service de jetons coupé reste une panne passagère : envoi en attente, rien d'incertain
#[tokio::test]
async fn regression_token_refresh_cut_off_is_transient_not_an_uncertain_send() {
    within(async {
        let mut fx = started().await;
        fx.credentials = FakeCredentials::stale_login_with_cut_token_service();
        let (_, action) = schedule_ready_draft(&fx, "k-cut-refresh");
        fx.advance(SEND_UNDO_DELAY_MILLIS);
        let mut client = fx.client().await;
        let report = fx.replay(&mut client).await;
        assert_eq!(report.uncertain, 0);
        let code = "network.unreachable".to_string();
        assert_eq!(report.stopped, Some(ReplayStop::Retry { code }));
        assert_eq!(state_of(&fx, &action), ActionState::Pending);
        assert!(fx.smtp.received().is_empty());
    })
    .await;
}

/// Un service de jetons trop lent (network.timeout) n'a rien envoyé : passager, jamais incertain
#[tokio::test]
async fn regression_token_refresh_timeout_is_transient_not_an_uncertain_send() {
    within(async {
        let mut fx = started().await;
        fx.credentials = FakeCredentials::stale_login_with_slow_token_service();
        let (_, action) = schedule_ready_draft(&fx, "k-slow-refresh");
        fx.advance(SEND_UNDO_DELAY_MILLIS);
        let mut client = fx.client().await;
        let report = fx.replay(&mut client).await;
        assert_eq!(report.uncertain, 0);
        let code = "network.unreachable".to_string();
        assert_eq!(report.stopped, Some(ReplayStop::Retry { code }));
        assert_eq!(state_of(&fx, &action), ActionState::Pending);
        assert!(fx.smtp.received().is_empty());
    })
    .await;
}
