#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Ajout et test de connexion d'un compte : le serveur SMTP est vérifié avec l'IMAP (AG-14).

mod support_api;

use std::time::Duration;

use mira_core::api::{AccountRequest, CredentialInput, Request, SecretInput};
use mira_vocabulary::{ErrorCategory, ServerSettings};
use support_api::endpoints;
use support_api::fixture::{self, Fixture};
use support_api::queries::accounts;
use support_api::requests::{add_request, manual_draft};
use tokio::time::timeout;

const LIMIT: Duration = Duration::from_secs(20);

/// Serveurs dont l'IMAP répond mais dont le SMTP n'écoute nulle part
fn smtp_unreachable(f: &Fixture) -> ServerSettings {
    endpoints::local_servers(f.imap.port(), 1)
}

/// Un SMTP injoignable fait refuser l'ajout : aucun compte, aucun secret rangé
#[tokio::test]
async fn regression_adding_an_account_with_unreachable_smtp_is_refused() {
    timeout(LIMIT, async {
        let f = fixture::start().await;
        f.credentials.add_password("carol@other.test", "other-pass");
        let draft = manual_draft("carol@other.test", smtp_unreachable(&f), "fake_fallback");
        let request = add_request(draft, "other-pass", "key-smtp");
        let error = f.engine.execute(request).await.unwrap_err();
        assert_eq!(error.category, ErrorCategory::Network);
        assert!(accounts(&f.engine).await.is_empty());
        assert_eq!(f.secrets.entry_count(), 0);
    })
    .await
    .unwrap();
}

/// Le test de connexion signale lui aussi le SMTP injoignable
#[tokio::test]
async fn regression_connection_test_reports_unreachable_smtp() {
    timeout(LIMIT, async {
        let f = fixture::start().await;
        f.credentials.add_password("carol@other.test", "other-pass");
        let draft = manual_draft("carol@other.test", smtp_unreachable(&f), "fake_fallback");
        let request = Request::Account(AccountRequest::TestConnection {
            draft,
            credential: CredentialInput::Password(SecretInput::new("other-pass")),
        });
        let error = f.engine.execute(request).await.unwrap_err();
        assert_eq!(error.category, ErrorCategory::Network);
    })
    .await
    .unwrap();
}
