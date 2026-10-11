#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Coffre et rafraîchissement silencieux (7.7, S10) ; aucun secret en base (invariant 15).

mod support_auth;

use std::sync::Arc;

use mira_core::auth::CredentialVault;
use mira_core::ports::{MemorySecretStore, SecretValue};
use mira_core::storage::Database;
use mira_vocabulary::{AuthMethodKind, SecretKind, SuggestedAction};
use support_auth::flow::{approved_flow, clock};
use support_auth::vault::{fixture, ids};
use support_auth::{CLIENT_ID, LIMIT, USER, started_oauth};
use tokio::time::timeout;

/// Jeton expiré puis rafraîchi par le coffre : nouveau jeton rangé, jeton de rafraîchissement gardé
#[tokio::test]
async fn vault_refreshes_an_expired_access_token_silently() {
    timeout(LIMIT, async {
        let (server, credentials) = started_oauth().await;
        let (flow, tokens) = approved_flow(&server, 31).await;
        let fixture = fixture();
        let (profile, account) = ids();
        let vault = &fixture.vault;
        vault.store_tokens(&profile, &account, &tokens).unwrap();
        server.expire_access_tokens(USER);
        assert!(!credentials.check_access_token(USER, tokens.access_token.expose()));
        let url = flow.token_url();
        let clock = clock();
        let refreshed = vault.refresh_access_token(&profile, &account, url, CLIENT_ID, &clock);
        let fresh = refreshed.await.unwrap();
        assert!(credentials.check_access_token(USER, fresh.expose()));
        let login = vault.load_login(&profile, &account, AuthMethodKind::OAuth2, USER);
        assert_eq!(login.unwrap().secret.expose(), fresh.expose());
        let kept = vault.load_secret(&profile, &account, SecretKind::RefreshToken);
        let original = tokens.refresh_token.unwrap();
        assert_eq!(kept.unwrap().unwrap().expose(), original.expose());
    })
    .await
    .unwrap();
}

/// Jeton de rafraîchissement absent ou révoqué : token_revoked, le compte devra se réauthentifier
#[tokio::test]
async fn revoked_or_missing_refresh_token_requires_reauthentication() {
    timeout(LIMIT, async {
        let (server, _) = started_oauth().await;
        let (flow, tokens) = approved_flow(&server, 32).await;
        let fixture = fixture();
        let (profile, account) = ids();
        let (url, clock) = (flow.token_url(), clock());
        let vault = &fixture.vault;
        let missing = vault.refresh_access_token(&profile, &account, url, CLIENT_ID, &clock);
        assert_eq!(
            missing.await.unwrap_err().code,
            "authentication.token_revoked"
        );
        vault.store_tokens(&profile, &account, &tokens).unwrap();
        server.revoke_refresh_tokens(USER);
        let revoked = vault.refresh_access_token(&profile, &account, url, CLIENT_ID, &clock);
        let error = revoked.await.unwrap_err();
        assert_eq!(error.code, "authentication.token_revoked");
        assert_eq!(
            error.suggested_action,
            Some(SuggestedAction::Reauthenticate)
        );
    })
    .await
    .unwrap();
}

/// Invariant 15 : après stockage des secrets, aucun fichier de la base ne contient leur valeur
#[tokio::test]
async fn no_secret_value_reaches_the_database_files() {
    timeout(LIMIT, async {
        let (server, _) = started_oauth().await;
        let (_, tokens) = approved_flow(&server, 33).await;
        let dir = tempfile::tempdir().unwrap();
        let database = Arc::new(Database::open(&dir.path().join("mira.sqlite3")).unwrap());
        let vault = CredentialVault::new(Arc::new(MemorySecretStore::new()), database);
        let (profile, account) = ids();
        let password = SecretValue::new("pw-Tres-Secret-31");
        vault.store_tokens(&profile, &account, &tokens).unwrap();
        vault.store_password(&profile, &account, &password).unwrap();
        let refresh = tokens.refresh_token.as_ref().unwrap();
        let needles = [
            tokens.access_token.expose(),
            refresh.expose(),
            password.expose(),
        ];
        for entry in std::fs::read_dir(dir.path()).unwrap() {
            let bytes = std::fs::read(entry.unwrap().path()).unwrap();
            for needle in needles {
                assert!(!bytes.windows(needle.len()).any(|w| w == needle.as_bytes()));
            }
        }
    })
    .await
    .unwrap();
}
