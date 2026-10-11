//! Comptes et identités de test.

use mira_core::storage::Database;
use mira_vocabulary::{
    Account, AccountId, AccountState, AuthMethodKind, EmailAddress, Identity, IdentityId,
    ProfileId, ProviderId, Security, ServerEndpoint, ServerSettings, SyncSettings, Timestamp,
};

/// Compte de test à l'adresse alice@example.org
pub fn sample_account(id: &str) -> Account {
    let endpoint = |host: &str, port: u16| ServerEndpoint {
        host: host.to_string(),
        port,
        security: Security::Tls,
    };
    Account {
        id: AccountId::from_raw(id),
        profile: ProfileId::from_raw("default"),
        address: EmailAddress::parse("alice@example.org").unwrap(),
        display_name: "Alice".to_string(),
        provider: ProviderId::new("fake"),
        auth: AuthMethodKind::AppPassword,
        servers: Some(ServerSettings {
            imap: endpoint("imap.example.org", 993),
            smtp: endpoint("smtp.example.org", 465),
        }),
        sync: SyncSettings::default(),
        state: AccountState::Connected,
        created_at: Timestamp::from_millis(1_000),
    }
}

/// Identité de test d'un compte
pub fn sample_identity(id: &str, account: &str, is_default: bool) -> Identity {
    Identity {
        id: IdentityId::from_raw(id),
        account: AccountId::from_raw(account),
        address: EmailAddress::parse(&format!("{id}@example.org")).unwrap(),
        display_name: format!("Identité {id}"),
        signature: Some("-- \nAlice".to_string()),
        is_default,
    }
}

/// Base en mémoire contenant déjà le compte donné
pub fn database_with_account(id: &str) -> Database {
    let db = Database::open_in_memory().unwrap();
    db.write(|tx| tx.insert_account(&sample_account(id)))
        .unwrap();
    db
}
