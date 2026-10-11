#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Flux OAuth2 par la façade : annulation, refus, `state` falsifié, réauthentification, erreurs (15.2, 7.7).

#[path = "support_auth/browser.rs"]
mod browser;
mod support_api;

use mira_core::api::{AuthRequest, Request, Response};
use mira_vocabulary::SecretKind;
use support_api::fixture;
use support_api::oauth::{OAUTH_USER, add_oauth_account, authorize, reauthenticate, start_oauth};
use support_api::oauth_requests::{add_oauth_request, complete_request};

/// Un `state` falsifié dans la redirection est refusé et le flux est clos
#[tokio::test]
async fn forged_state_in_redirect_is_refused() {
    let f = fixture::start().await;
    let started = start_oauth(&f.engine, OAUTH_USER).await;
    let redirect = browser::query_value(&started.authorize_url, "redirect_uri").unwrap();
    let forged = browser::get(&redirect, "/callback?state=falsifie&code=x");
    let (_, outcome) = tokio::join!(forged, f.engine.execute(complete_request(&started.flow)));
    assert_eq!(
        outcome.unwrap_err().code,
        "authentication.oauth_state_mismatch"
    );
    let again = f
        .engine
        .execute(complete_request(&started.flow))
        .await
        .unwrap_err();
    assert_eq!(again.code, "invalid_input.parameter");
    assert_eq!(f.secrets.entry_count(), 0);
}

/// Un refus de l'utilisateur donne `oauth_cancelled` et ne range aucun jeton
#[tokio::test]
async fn user_denial_is_reported_as_cancellation() {
    let f = fixture::start().await;
    let started = start_oauth(&f.engine, OAUTH_USER).await;
    let (denied, outcome) = tokio::join!(
        f.oauth.deny(&started.authorize_url),
        f.engine.execute(complete_request(&started.flow))
    );
    denied.unwrap();
    assert_eq!(outcome.unwrap_err().code, "authentication.oauth_cancelled");
    assert_eq!(f.secrets.entry_count(), 0);
}

/// Annuler pendant l'attente interrompt `CompleteOAuth` ; annuler deux fois reste sans erreur
#[tokio::test]
async fn cancel_interrupts_a_waiting_completion() {
    let f = fixture::start().await;
    let started = start_oauth(&f.engine, OAUTH_USER).await;
    let cancel = Request::Auth(AuthRequest::CancelOAuth {
        flow: started.flow.clone(),
    });
    let waiting = f.engine.execute(complete_request(&started.flow));
    let cancelling = async {
        tokio::task::yield_now().await;
        f.engine.execute(cancel.clone()).await
    };
    let (outcome, cancelled) = tokio::join!(waiting, cancelling);
    assert_eq!(outcome.unwrap_err().code, "authentication.oauth_cancelled");
    assert_eq!(cancelled.unwrap(), Response::Done);
    assert_eq!(f.engine.execute(cancel).await.unwrap(), Response::Done);
}

/// Annuler un flux terminé mais pas encore utilisé efface les jetons déjà rangés
#[tokio::test]
async fn cancel_after_completion_discards_the_tokens() {
    let f = fixture::start().await;
    let flow = authorize(&f, OAUTH_USER).await;
    assert_eq!(f.secrets.entry_count(), 2);
    let cancel = Request::Auth(AuthRequest::CancelOAuth { flow: flow.clone() });
    f.engine.execute(cancel).await.unwrap();
    assert_eq!(f.secrets.entry_count(), 0);
    let error = f
        .engine
        .execute(add_oauth_request(&flow, "k"))
        .await
        .unwrap_err();
    assert_eq!(error.code, "invalid_input.parameter");
}

/// Démarrer un flux est refusé si le provider n'offre pas OAuth2 ou si l'application n'est pas configurée
#[tokio::test]
async fn start_oauth_refuses_unsupported_provider_or_missing_client_id() {
    let f = fixture::start().await;
    let labels = Request::Auth(AuthRequest::StartOAuth {
        address: fixture::LABELS_USER.to_string(),
        provider: None,
    });
    assert_eq!(
        f.engine.execute(labels).await.unwrap_err().code,
        "invalid_input.parameter"
    );
    let home = tempfile::tempdir().unwrap();
    let bare = support_api::open(home.path());
    let oauth = Request::Auth(AuthRequest::StartOAuth {
        address: OAUTH_USER.to_string(),
        provider: None,
    });
    assert_eq!(
        bare.execute(oauth).await.unwrap_err().code,
        "invalid_input.parameter"
    );
}

/// Réauthentifier un compte OAuth2 remplace ses jetons ; un compte à mot de passe est refusé
#[tokio::test]
async fn reauthenticate_replaces_tokens_and_refuses_password_accounts() {
    let f = fixture::start().await;
    let account = add_oauth_account(&f, "k").await;
    let old = f
        .stored_secret(&account.id, SecretKind::AccessToken)
        .unwrap();
    let again = reauthenticate(&f, &account.id).await;
    assert_ne!(
        f.stored_secret(&account.id, SecretKind::AccessToken)
            .unwrap(),
        old
    );
    assert_eq!(
        f.engine
            .execute(complete_request(&again))
            .await
            .unwrap_err()
            .code,
        "invalid_input.parameter"
    );
    let password = f.add_folders_account("p").await;
    let request = Request::Auth(AuthRequest::Reauthenticate { account: password });
    assert_eq!(
        f.engine.execute(request).await.unwrap_err().code,
        "invalid_input.parameter"
    );
}
