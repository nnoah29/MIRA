#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Effacement mémoire des secrets (audit SEC, A02-11) : `SecretValue` s'efface sur demande et à sa destruction.

use mira_core::ports::SecretValue;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Échoue à la compilation si le type ne s'efface pas à sa destruction
fn assert_zeroize_on_drop<T: ZeroizeOnDrop>() {}

/// Le secret annonce qu'il s'efface à sa destruction
#[test]
fn secret_value_is_zeroized_on_drop() {
    assert_zeroize_on_drop::<SecretValue>();
}

/// Un effacement demandé vide la valeur lisible
#[test]
fn zeroize_clears_the_exposed_value() {
    let mut secret = SecretValue::new("pw-correct-horse-9");
    assert_eq!(secret.expose(), "pw-correct-horse-9");
    secret.zeroize();
    assert_eq!(secret.expose(), "");
}

/// Une copie s'efface sans toucher l'original
#[test]
fn zeroizing_a_clone_leaves_the_original() {
    let original = SecretValue::new("abc");
    let mut copy = original.clone();
    copy.zeroize();
    assert_eq!(original.expose(), "abc");
    assert_eq!(copy.expose(), "");
}
