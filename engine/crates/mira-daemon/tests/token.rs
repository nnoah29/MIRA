#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Jeton local : aléa, permissions du fichier, comparaison exacte, aucune fuite par `Debug`.

use std::os::unix::fs::PermissionsExt;

use mira_daemon::token::Token;

/// Un jeton neuf fait 64 caractères hexadécimaux et deux fichiers donnent deux jetons.
#[test]
fn a_new_token_is_64_hex_characters_and_two_files_differ() {
    let dir = tempfile::tempdir().unwrap();
    let first = Token::load_or_create(&dir.path().join("a.token")).unwrap();
    let second = Token::load_or_create(&dir.path().join("b.token")).unwrap();
    assert_eq!(first.as_str().len(), 64);
    assert!(first.as_str().bytes().all(|b| b.is_ascii_hexdigit()));
    assert_ne!(first.as_str(), second.as_str());
}

/// Le fichier du jeton est créé en 0600, dossier parent compris s'il manque.
#[test]
fn the_token_file_is_mode_0600() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested/daemon.token");
    Token::load_or_create(&path).unwrap();
    let mode = std::fs::metadata(&path).unwrap().permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
}

/// Recharger le même fichier rend le même jeton.
#[test]
fn loading_twice_returns_the_same_token() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("daemon.token");
    let first = Token::load_or_create(&path).unwrap();
    let again = Token::load_or_create(&path).unwrap();
    assert_eq!(first.as_str(), again.as_str());
}

/// Un fichier lisible par d'autres que son propriétaire est refusé.
#[test]
fn a_file_readable_by_others_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("daemon.token");
    Token::load_or_create(&path).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    let error = Token::load_or_create(&path).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
}

/// Seul le jeton exact est accepté : ni vide, ni préfixe, ni plus long, ni autre casse.
#[test]
fn matches_accepts_only_the_exact_token() {
    let dir = tempfile::tempdir().unwrap();
    let token = Token::load_or_create(&dir.path().join("t")).unwrap();
    let exact = token.as_str().to_string();
    assert!(token.matches(&exact));
    assert!(!token.matches(""));
    assert!(!token.matches(&exact[..63]));
    assert!(!token.matches(&format!("{exact}0")));
    assert!(!token.matches(&exact.to_uppercase()));
}

/// `Debug` ne montre jamais le jeton.
#[test]
fn debug_never_shows_the_token() {
    let dir = tempfile::tempdir().unwrap();
    let token = Token::load_or_create(&dir.path().join("t")).unwrap();
    assert!(!format!("{token:?}").contains(token.as_str()));
}
