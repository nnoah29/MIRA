//! Monde avec un compte IMAP de test branché sur le faux serveur du testkit (boucle locale).

use mira_assembly::accounts::NetworkPolicy;
use mira_assembly::api::{
    AccountDraft, AccountRequest, CredentialInput, Engine, Request, Response, SecretInput,
};
use mira_assembly::mira_vocabulary::{
    AccountId, AuthMethodKind, IdempotencyKey, Security, ServerEndpoint, ServerSettings,
};
use mira_testkit::Credentials;
use mira_testkit::imap::{CapabilitySet, FakeImapServer, FakeMailStore};

use super::world::TestWorld;

pub const USER: &str = "alice@folders.test";
pub const PASSWORD: &str = "pa55-SECRET-xyz";

/// Réseau de test : le domaine `folders.test` vise le faux serveur, en clair.
fn network_for(imap_port: u16) -> NetworkPolicy {
    let endpoint = |port: u16| ServerEndpoint {
        host: "127.0.0.1".to_string(),
        port,
        security: Security::Plaintext,
    };
    let settings = ServerSettings {
        imap: endpoint(imap_port),
        smtp: endpoint(1),
    };
    let mut network = NetworkPolicy::default();
    network
        .server_overrides
        .insert("folders.test".to_string(), settings);
    network
}

/// Enregistre le compte de test par le contrat du moteur et rend son identifiant.
pub(super) async fn add_test_account(engine: &Engine) -> Option<AccountId> {
    let draft = AccountDraft {
        address: USER.to_string(),
        display_name: USER.to_string(),
        provider: None,
        servers: None,
        auth: AuthMethodKind::Password,
    };
    let response = engine
        .execute(Request::Account(AccountRequest::AddAccount {
            draft,
            credential: CredentialInput::Password(SecretInput::new(PASSWORD)),
            idempotency_key: IdempotencyKey("tui-test-account".to_string()),
        }))
        .await
        .unwrap();
    match response {
        Response::Account(account) => Some(account.id),
        _ => None,
    }
}

impl TestWorld {
    /// Monde avec un faux serveur IMAP et le compte `alice@folders.test` déjà ajouté.
    pub async fn with_imap_account() -> Self {
        let credentials = Credentials::default();
        credentials.add_password(USER, PASSWORD);
        let imap = FakeImapServer::start(FakeMailStore::new(), credentials, CapabilitySet::full())
            .await
            .unwrap();
        let network = network_for(imap.port());
        let home = tempfile::tempdir().unwrap();
        let mut world = Self::open_in(home, network, Some(imap));
        world.account = add_test_account(&world.engine).await;
        world
    }

    /// Monde avec un faux serveur IMAP prêt pour `USER` mais sans compte au moteur.
    pub async fn with_empty_imap() -> Self {
        let credentials = Credentials::default();
        credentials.add_password(USER, PASSWORD);
        let imap = FakeImapServer::start(FakeMailStore::new(), credentials, CapabilitySet::full())
            .await
            .unwrap();
        let mut network = network_for(imap.port());
        network.background_sync = mira_assembly::sync::BackgroundSync::On(
            mira_assembly::sync::SupervisorConfig::default(),
        );
        let home = tempfile::tempdir().unwrap();
        Self::open_in(home, network, Some(imap))
    }
}
