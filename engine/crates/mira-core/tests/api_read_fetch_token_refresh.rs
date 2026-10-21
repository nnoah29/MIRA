#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Téléchargement explicite du corps d'un compte OAuth2 dont le jeton d'accès n'est plus valable (7.7, S10).

mod support;
mod support_api;
mod support_mirror;
mod support_read;

use mira_core::api::{MessageRequest, Request, Response};
use mira_core::operations::OperationStatus;
use mira_core::storage::Database;
use support_api::fixture::{self, Fixture};
use support_api::oauth::{OAUTH_USER, add_oauth_account};
use support_read::calls::{operation_of, run, wait_finished};
use support_read::mail;
use support_read::seed::{add_inbox, add_message_with_header};

/// Dépose un message sur le serveur et le déclare, en-tête seul, dans le miroir du compte OAuth
async fn oauth_message_awaiting_body(f: &Fixture) -> mira_vocabulary::MessageId {
    let account = add_oauth_account(f, "key-fetch-refresh").await.id;
    let path = f.home.path().join("profiles/default/mira.sqlite3");
    let mirror = Database::open(&path).unwrap();
    let mailbox = add_inbox(&mirror, &account);
    let raw = mail::plain("licorne", "Le corps contient un narval.");
    let uid = f.store.deliver(OAUTH_USER, "INBOX", &raw, &[]);
    let header = mail::message_id_of(&raw);
    add_message_with_header(&mirror, &account, &mailbox, "m1", uid, 1_000, &header)
}

/// Lance FetchMessageBody et rend l'état final de l'opération
async fn fetch_body(f: &Fixture, message: &mira_vocabulary::MessageId) -> OperationStatus {
    let request = Request::Message(MessageRequest::FetchMessageBody {
        message: message.clone(),
    });
    let operation = operation_of(run(&f.engine, request).await.unwrap());
    wait_finished(&f.engine, &operation).await
}

/// Un jeton révoqué par le serveur est rafraîchi en silence : le corps est téléchargé
#[tokio::test]
async fn regression_fetch_body_refreshes_a_rejected_access_token() {
    let f = fixture::start().await;
    let id = oauth_message_awaiting_body(&f).await;
    f.oauth.expire_access_tokens(OAUTH_USER);
    let before = f.oauth.token_requests();
    assert_eq!(fetch_body(&f, &id).await, OperationStatus::Finished);
    assert_eq!(f.oauth.token_requests(), before + 1);
    let body = Request::Message(MessageRequest::GetMessageBody { message: id });
    let Response::MessageBody(view) = run(&f.engine, body).await.unwrap() else {
        panic!("réponse")
    };
    assert!(view.text.contains("narval"));
}

/// Un jeton encore valable ne déclenche aucun rafraîchissement
#[tokio::test]
async fn fetch_body_with_a_valid_token_does_not_refresh() {
    let f = fixture::start().await;
    let id = oauth_message_awaiting_body(&f).await;
    let before = f.oauth.token_requests();
    assert_eq!(fetch_body(&f, &id).await, OperationStatus::Finished);
    assert_eq!(f.oauth.token_requests(), before);
}
