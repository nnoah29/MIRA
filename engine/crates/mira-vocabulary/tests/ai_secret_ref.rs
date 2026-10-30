#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Clé de stockage des références de secret d'IA.

use mira_vocabulary::{
    AccountId, AiConnectionId, AiSecretRef, ProfileId, SecretKind, SecretLocation, SecretRef,
};

/// La clé d'une clé d'API d'IA suit le format `mira/<profil>/ai/<connexion>/api_key`.
#[test]
fn ai_secret_ref_storage_key_format() {
    let reference = AiSecretRef {
        profile: ProfileId::from_raw("p1"),
        connection: AiConnectionId::from_raw("c1"),
    };
    assert_eq!(reference.storage_key(), "mira/p1/ai/c1/api_key");
}

/// Aucune clé de compte ne peut coïncider avec une clé d'IA, même avec un compte nommé « ai ».
#[test]
fn ai_and_account_keys_never_collide() {
    let ai = AiSecretRef {
        profile: ProfileId::from_raw("p1"),
        connection: AiConnectionId::from_raw("c1"),
    };
    let kinds = [
        SecretKind::Password,
        SecretKind::AccessToken,
        SecretKind::RefreshToken,
    ];
    for kind in kinds {
        let account = SecretRef {
            profile: ProfileId::from_raw("p1"),
            account: AccountId::from_raw("ai"),
            kind,
        };
        assert_ne!(SecretLocation::storage_key(&account), ai.storage_key());
    }
}

/// La clé d'un secret de compte reste identique à son format historique.
#[test]
fn account_secret_key_is_unchanged_through_the_trait() {
    let account = SecretRef {
        profile: ProfileId::from_raw("p1"),
        account: AccountId::from_raw("a1"),
        kind: SecretKind::AccessToken,
    };
    let location: &dyn SecretLocation = &account;
    assert_eq!(location.storage_key(), "mira/p1/a1/access_token");
}
