#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use mira_vocabulary::{
    Account, AccountId, AccountState, ActionKind, AuthMethod, AuthMethodKind, EmailAddress,
    MailboxId, MailboxRole, ProfileId, ProviderId, SecretKind, SecretRef, SyncSettings, Timestamp,
};

/// La clé de stockage d'un secret contient le profil, le compte et la nature.
#[test]
fn secret_ref_storage_key_contains_profile_and_account() {
    let secret = SecretRef {
        profile: ProfileId::from_raw("p1"),
        account: AccountId::from_raw("a1"),
        kind: SecretKind::RefreshToken,
    };
    assert_eq!(secret.storage_key(), "mira/p1/a1/refresh_token");
}

/// Seules Inbox, Sent, Drafts, Trash et Junk sont des boîtes système (9.9).
#[test]
fn only_standard_mailbox_roles_are_system() {
    let system = [
        MailboxRole::Inbox,
        MailboxRole::Sent,
        MailboxRole::Drafts,
        MailboxRole::Trash,
        MailboxRole::Junk,
    ];
    let other = [
        MailboxRole::Archive,
        MailboxRole::All,
        MailboxRole::Flagged,
        MailboxRole::Other,
    ];
    assert!(system.iter().all(MailboxRole::is_system));
    assert!(other.iter().all(|role| !role.is_system()));
}

/// Les réglages de synchronisation par défaut couvrent 90 jours et toutes les boîtes (D-21).
#[test]
fn sync_settings_default_to_ninety_days() {
    let settings = SyncSettings::default();
    assert_eq!(settings.window_days, 90);
    assert_eq!(settings.prefetch_bodies, 50);
    assert_eq!(settings.synced_mailboxes, None);
}

/// Une action sérialisée porte son type en `snake_case` et ses paramètres.
#[test]
fn action_kind_serializes_with_snake_case_type_tag() {
    let action = ActionKind::Move {
        to: MailboxId::from_raw("m1"),
    };
    let json = serde_json::to_value(&action).unwrap();
    assert_eq!(json, serde_json::json!({"type": "move", "to": "m1"}));
    let archive = serde_json::to_value(ActionKind::Archive).unwrap();
    assert_eq!(archive, serde_json::json!({"type": "archive"}));
    let back: ActionKind = serde_json::from_value(json).unwrap();
    assert_eq!(back, action);
}

/// La méthode OAuth2 se nomme `oauth2` en JSON et un compte fait l'aller-retour JSON.
#[test]
fn account_round_trips_through_json() {
    let account = Account {
        id: AccountId::from_raw("a1"),
        profile: ProfileId::from_raw("p1"),
        address: EmailAddress::parse("alice@example.com").unwrap(),
        display_name: "Alice".to_string(),
        provider: ProviderId::new("generic"),
        auth: AuthMethodKind::OAuth2,
        servers: None,
        sync: SyncSettings::default(),
        state: AccountState::Error {
            code: "network.offline".to_string(),
        },
        created_at: Timestamp::from_millis(1_700_000_000_000),
    };
    let json = serde_json::to_value(&account).unwrap();
    assert_eq!(json["auth"], "oauth2");
    assert_eq!(json["state"]["type"], "error");
    let back: Account = serde_json::from_value(json).unwrap();
    assert_eq!(back, account);
    assert_eq!(AuthMethod::Password.kind(), AuthMethodKind::Password);
}
