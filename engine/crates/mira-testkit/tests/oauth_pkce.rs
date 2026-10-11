#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! PKCE S256 (RFC 7636) : calcul du défi et vérification du vérificateur.

use mira_testkit::oauth::pkce::{challenge_for, is_valid_verifier, verifies};

const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
const CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

/// Le défi correspond au vecteur de l'annexe B de la RFC 7636.
#[test]
fn challenge_matches_the_rfc_7636_vector() {
    assert_eq!(challenge_for(VERIFIER), CHALLENGE);
}

/// Un vérificateur correspondant au défi est accepté, un autre est refusé.
#[test]
fn matching_verifier_is_accepted_and_other_is_refused() {
    assert!(verifies(VERIFIER, CHALLENGE));
    assert!(!verifies(&"a".repeat(43), CHALLENGE));
}

/// La longueur du vérificateur doit être comprise entre 43 et 128 caractères.
#[test]
fn verifier_length_must_be_between_43_and_128() {
    assert!(!is_valid_verifier(&"a".repeat(42)));
    assert!(is_valid_verifier(&"a".repeat(43)));
    assert!(is_valid_verifier(&"a".repeat(128)));
    assert!(!is_valid_verifier(&"a".repeat(129)));
}

/// Seuls les caractères non réservés sont permis dans le vérificateur.
#[test]
fn verifier_only_allows_unreserved_characters() {
    assert!(is_valid_verifier(&format!("{}-._~", "a".repeat(40))));
    assert!(!is_valid_verifier(&format!("{}+/=", "a".repeat(40))));
}
