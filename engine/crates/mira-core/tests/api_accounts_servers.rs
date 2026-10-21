#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Changer les serveurs d'un compte exige une connexion réussie avec le secret rangé (revue 29).

mod support_api;

use std::time::Duration;

use mira_core::api::{AccountChanges, AccountRequest, Request};
use mira_testkit::smtp::FakeSmtpServer;
use mira_vocabulary::{Account, AccountId, ErrorCategory, ServerSettings};
use support_api::endpoints;
use support_api::fixture::{self, Fixture};
use support_api::queries::accounts;
use support_api::requests::{account_response, add_account, manual_draft};
use tokio::time::timeout;

const LIMIT: Duration = Duration::from_secs(20);

/// Ajoute un compte « Autre » dont les serveurs sont saisis à la main
async fn add_other_account(f: &Fixture) -> Account {
    f.credentials.add_password("carol@other.test", "other-pass");
    let servers = endpoints::local_servers(f.imap.port(), f.smtp.port());
    let draft = manual_draft("carol@other.test", servers, "fake_fallback");
    add_account(&f.engine, draft, "other-pass", "key-other").await
}

/// Requête de modification des seuls serveurs
fn servers_request(account: &AccountId, servers: ServerSettings) -> AccountRequest {
    AccountRequest::UpdateAccount {
        account: account.clone(),
        changes: AccountChanges {
            servers: Some(servers),
            ..AccountChanges::default()
        },
    }
}

/// Des serveurs injoignables sont refusés et l'ancienne configuration reste en place
#[tokio::test]
async fn regression_unreachable_servers_are_refused_and_nothing_changes() {
    timeout(LIMIT, async {
        let f = fixture::start().await;
        let account = add_other_account(&f).await;
        let request =
            Request::Account(servers_request(&account.id, endpoints::local_servers(1, 1)));
        let error = f.engine.execute(request).await.unwrap_err();
        assert_eq!(error.category, ErrorCategory::Network);
        assert_eq!(accounts(&f.engine).await, vec![account]);
    })
    .await
    .unwrap();
}

/// Des serveurs qui acceptent le secret rangé sont enregistrés
#[tokio::test]
async fn reachable_servers_accepting_the_stored_secret_are_saved() {
    timeout(LIMIT, async {
        let f = fixture::start().await;
        let account = add_other_account(&f).await;
        let other_smtp = FakeSmtpServer::start(f.credentials.clone()).await.unwrap();
        let servers = endpoints::local_servers(f.imap.port(), other_smtp.port());
        let request = servers_request(&account.id, servers.clone());
        let updated = account_response(&f.engine, request).await;
        assert_eq!(updated.servers, Some(servers));
    })
    .await
    .unwrap();
}
