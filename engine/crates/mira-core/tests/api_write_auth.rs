#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Authentification pendant le rejeu : jeton expiré rafraîchi en silence, révoqué, mot de passe refusé, compte en pause (S10, 7.7).

mod support;
mod support_api;
mod support_imap;
mod support_mirror;
mod support_write;

use mira_core::api::{AccountRequest, AuthRequest, Request, SecretInput};
use mira_vocabulary::{AccountState, ActionKind, ActionState};
use support_api::oauth::{OAUTH_USER, reauthenticate};
use support_api::queries::state_changes;
use support_imap::within;
use support_write::fixture::Fixture;
use support_write::oauth;
use support_write::timing::settle;
use support_write::waits::eventually;
use support_write::{PASSWORD, USER};

/// S10 : jeton expiré et révoqué, l'action reste en file, le compte demande une authentification, puis le rejeu reprend
#[tokio::test]
async fn scenario_s10_token_expired_keeps_actions_queued() {
    within(async {
        let b = oauth::start().await;
        b.api.oauth.revoke_refresh_tokens(OAUTH_USER);
        let action = b.mark_read("msg-1", "k1").await;
        b.clock.advance_millis(3_700_000);
        eventually("authentification requise", || {
            b.account_state() == AccountState::AuthRequired
        })
        .await;
        assert_eq!(b.state_of(&action), ActionState::Pending);
        assert!(!b.server_seen(1));
        reauthenticate(&b.api, &b.account).await;
        eventually("action rejouée", || {
            b.state_of(&action) == ActionState::Succeeded
        })
        .await;
        assert!(b.server_seen(1));
    })
    .await;
}

/// Jeton expiré mais rafraîchissable : le rejeu le renouvelle en silence, le compte reste connecté
#[tokio::test]
async fn expired_token_is_refreshed_silently_during_replay() {
    within(async {
        let b = oauth::start().await;
        let before = b.api.oauth.token_requests();
        b.api.oauth.expire_access_tokens(OAUTH_USER);
        let action = b.mark_read("msg-2", "k1").await;
        b.clock.advance_millis(3_700_000);
        eventually("action rejouée", || {
            b.state_of(&action) == ActionState::Succeeded
        })
        .await;
        assert!(b.server_seen(2));
        assert!(b.api.oauth.token_requests() > before);
        let states = state_changes(&b.api.engine).await;
        assert!(!states.contains(&AccountState::AuthRequired));
    })
    .await;
}

/// Mot de passe refusé par le serveur : action en file, compte à réauthentifier, reprise après le nouveau mot de passe
#[tokio::test]
async fn refused_password_requires_authentication_then_replay_resumes() {
    within(async {
        let f = Fixture::start().await;
        f.credentials.add_password(USER, "nouveau-mot-de-passe-9");
        let ids = f
            .apply(ActionKind::MarkRead { read: true }, &["msg-1"], "k1")
            .await;
        f.advance(5_001);
        eventually("authentification requise", || {
            f.account_state() == AccountState::AuthRequired
        })
        .await;
        assert_eq!(f.state_of(&ids[0]), ActionState::Pending);
        let store = AuthRequest::StoreAppPassword {
            account: f.account.clone(),
            password: SecretInput::new("nouveau-mot-de-passe-9"),
        };
        f.run(Request::Auth(store)).await;
        f.wait_for_state(&ids[0], ActionState::Succeeded).await;
        assert!(f.server_seen("INBOX", 1));
        assert_ne!(PASSWORD, "nouveau-mot-de-passe-9");
    })
    .await;
}

/// Un compte en pause n'est pas rejoué ; à la reprise, la file repart
#[tokio::test]
async fn paused_account_is_not_replayed_until_resumed() {
    within(async {
        let f = Fixture::start().await;
        let pause = AccountRequest::PauseAccount {
            account: f.account.clone(),
        };
        f.run(Request::Account(pause)).await;
        let ids = f
            .apply(ActionKind::MarkFlagged { flagged: true }, &["msg-1"], "k1")
            .await;
        f.advance(5_001);
        settle().await;
        assert_eq!(f.state_of(&ids[0]), ActionState::Pending);
        assert_eq!(f.imap.connection_count(), 1);
        let resume = AccountRequest::ResumeAccount {
            account: f.account.clone(),
        };
        f.run(Request::Account(resume)).await;
        f.wait_for_state(&ids[0], ActionState::Succeeded).await;
    })
    .await;
}
