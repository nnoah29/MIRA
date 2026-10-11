#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Coffre d'identifiants : mots de passe, jetons, expiration et suppression (7.7).

mod support_auth;

use std::sync::Arc;

use mira_core::auth::CredentialVault;
use mira_core::ports::{MemorySecretStore, SecretStore, SecretValue};
use mira_core::smtp::SmtpCredential;
use mira_vocabulary::{AccountId, AuthMethodKind, SecretKind, Timestamp};
use support_auth::vault::{fixture, ids, keyring_value, token_set};

const USER: &str = "alice@example.org";

/// Mot de passe : rangé dans le trousseau, relu pour la connexion, jamais affiché
#[test]
fn password_is_stored_in_the_keyring_and_reloaded() {
    let fixture = fixture();
    let (profile, account) = ids();
    let password = SecretValue::new("pw-correct-horse-9");
    let vault = &fixture.vault;
    vault.store_password(&profile, &account, &password).unwrap();
    let stored = keyring_value(&fixture, SecretKind::Password);
    assert_eq!(stored.as_deref(), Some("pw-correct-horse-9"));
    let method = AuthMethodKind::AppPassword;
    let login = vault.load_login(&profile, &account, method, USER).unwrap();
    assert_eq!(login.secret.expose(), "pw-correct-horse-9");
    assert!(!format!("{login:?}").contains("pw-correct-horse-9"));
    let smtp = login.into_smtp_login();
    assert!(matches!(smtp.credential, SmtpCredential::Password(_)));
}

/// Jetons : secrets en trousseau, expiration en base, XOAUTH2 à la lecture
#[test]
fn tokens_go_to_the_keyring_and_expiry_to_the_database() {
    let fixture = fixture();
    let (profile, account) = ids();
    let tokens = token_set("at-1", Some("rt-1"), Some(5_000));
    let vault = &fixture.vault;
    vault.store_tokens(&profile, &account, &tokens).unwrap();
    let access = keyring_value(&fixture, SecretKind::AccessToken);
    let refresh = keyring_value(&fixture, SecretKind::RefreshToken);
    assert_eq!(access.as_deref(), Some("at-1"));
    assert_eq!(refresh.as_deref(), Some("rt-1"));
    assert_eq!(fixture.expiry(), Some(Timestamp::from_millis(5_000)));
    let method = AuthMethodKind::OAuth2;
    let login = vault.load_login(&profile, &account, method, USER).unwrap();
    assert_eq!(login.secret.expose(), "at-1");
    let smtp = login.into_smtp_login();
    assert!(matches!(smtp.credential, SmtpCredential::XOAuth2(_)));
}

/// Un rafraîchissement sans nouveau jeton de rafraîchissement conserve l'ancien
#[test]
fn storing_tokens_without_refresh_token_keeps_the_previous_one() {
    let fixture = fixture();
    let (profile, account) = ids();
    let first = token_set("at-1", Some("rt-1"), Some(1_000));
    let second = token_set("at-2", None, None);
    let vault = &fixture.vault;
    vault.store_tokens(&profile, &account, &first).unwrap();
    vault.store_tokens(&profile, &account, &second).unwrap();
    let refresh = keyring_value(&fixture, SecretKind::RefreshToken);
    assert_eq!(refresh.as_deref(), Some("rt-1"));
    let access = keyring_value(&fixture, SecretKind::AccessToken);
    assert_eq!(access.as_deref(), Some("at-2"));
    assert_eq!(fixture.expiry(), None);
}

/// L'expiration tient compte d'une marge d'une minute ; inconnue, elle n'est pas expirée
#[test]
fn expiry_check_uses_a_one_minute_margin() {
    let fixture = fixture();
    let (profile, account) = ids();
    let now = Timestamp::from_millis(1_000_000);
    let vault = &fixture.vault;
    assert!(!vault.access_token_expired(&account, now).unwrap());
    let tokens = token_set("at", None, Some(1_000_000 + 3_600_000));
    vault.store_tokens(&profile, &account, &tokens).unwrap();
    let check = |offset: i64| vault.access_token_expired(&account, now.plus_millis(offset));
    assert!(!check(0).unwrap() && !check(3_539_999).unwrap());
    assert!(check(3_540_000).unwrap() && check(4_000_000).unwrap());
}

/// delete_all efface les secrets et l'expiration du compte, sans toucher aux autres comptes
#[test]
fn delete_all_removes_every_secret_of_the_account_only() {
    let fixture = fixture();
    let (profile, account) = ids();
    let other = AccountId::from_raw("acc-2");
    let tokens = token_set("at", Some("rt"), Some(9));
    let vault = &fixture.vault;
    for id in [&account, &other] {
        vault
            .store_password(&profile, id, &SecretValue::new("pw"))
            .unwrap();
        vault.store_tokens(&profile, id, &tokens).unwrap();
    }
    vault.delete_all(&profile, &account).unwrap();
    for kind in [
        SecretKind::Password,
        SecretKind::AccessToken,
        SecretKind::RefreshToken,
    ] {
        let load = |id: &AccountId| vault.load_secret(&profile, id, kind).unwrap();
        assert!(load(&account).is_none() && load(&other).is_some());
    }
    assert_eq!(fixture.expiry(), None);
    assert!(
        fixture
            .database
            .read(|tx| tx.token_expiry(&other))
            .unwrap()
            .is_some()
    );
}

/// Secret absent : secrets.secret_missing ; trousseau indisponible : son erreur remonte
#[test]
fn missing_secret_and_unavailable_keyring_are_reported() {
    let fixture = fixture();
    let (profile, account) = ids();
    let method = AuthMethodKind::Password;
    let missing = fixture.vault.load_login(&profile, &account, method, "u");
    assert_eq!(missing.unwrap_err().code, "secrets.secret_missing");
    let broken: Arc<dyn SecretStore> = Arc::new(MemorySecretStore::unavailable());
    let vault = CredentialVault::new(broken, fixture.database.clone());
    let stored = vault.store_password(&profile, &account, &SecretValue::new("x"));
    assert_eq!(stored.unwrap_err().code, "secrets.keyring_unavailable");
    let read = vault.load_login(&profile, &account, AuthMethodKind::OAuth2, "u");
    assert_eq!(read.unwrap_err().code, "secrets.keyring_unavailable");
}
