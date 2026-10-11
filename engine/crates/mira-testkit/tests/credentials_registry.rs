#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Registre d'identifiants partagé : vérification et absence de fuite.

use mira_testkit::Credentials;

/// Le registre accepte les bons identifiants et refuse les autres
#[test]
fn registry_checks_passwords_and_tokens() {
    let credentials = Credentials::default();
    credentials.add_password("alice", "s3cret");
    credentials.add_access_token("alice", "tok-1");
    assert!(credentials.check_password("alice", "s3cret"));
    assert!(!credentials.check_password("alice", "wrong"));
    assert!(credentials.check_access_token("alice", "tok-1"));
    credentials.revoke_access_token("tok-1");
    assert!(!credentials.check_access_token("alice", "tok-1"));
}

/// L'affichage de débogage ne révèle aucun secret
#[test]
fn debug_output_never_reveals_secrets() {
    let credentials = Credentials::default();
    credentials.add_password("alice", "s3cret-password");
    credentials.add_access_token("alice", "tok-secret");
    let shown = format!("{credentials:?}");
    assert!(!shown.contains("s3cret-password"));
    assert!(!shown.contains("tok-secret"));
}
