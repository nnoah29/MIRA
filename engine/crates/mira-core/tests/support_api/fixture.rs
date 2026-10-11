//! Moteur de test relié à un faux serveur IMAP : les domaines de test sont redirigés vers lui.

use std::sync::Arc;

use mira_core::api::{Engine, EngineConfig};
use mira_core::ports::{MemorySecretStore, SecretStore};
use mira_testkit::Credentials;
use mira_testkit::imap::{CapabilitySet, FakeImapServer, FakeMailStore};
use mira_testkit::oauth::FakeOAuthServer;
use mira_vocabulary::{
    AccountId, AuthMethodKind, CONTRACT_VERSION, ProfileId, SecretKind, SecretRef, Security,
    ServerEndpoint, ServerSettings,
};
use tempfile::TempDir;

use super::config_with_secrets;
use super::requests::{add_account, draft};

pub const FOLDERS_USER: &str = "alice@folders.test";
pub const FOLDERS_PASSWORD: &str = "pa55-SECRET-xyz";
pub const LABELS_USER: &str = "bob@labels.test";
pub const LABELS_PASSWORD: &str = "app-SECRET-789";

/// Moteur ouvert, trousseau et faux serveur IMAP où `folders.test` et `labels.test` sont redirigés.
pub struct Fixture {
    pub home: TempDir,
    pub engine: Engine,
    pub secrets: Arc<MemorySecretStore>,
    pub imap: FakeImapServer,
    pub store: FakeMailStore,
    pub oauth: FakeOAuthServer,
    pub credentials: Credentials,
}

impl Fixture {
    /// Ajoute le compte à mot de passe de `folders.test` avec la clé d'idempotence donnée
    pub async fn add_folders_account(&self, key: &str) -> AccountId {
        let draft = draft(FOLDERS_USER, AuthMethodKind::Password);
        add_account(&self.engine, draft, FOLDERS_PASSWORD, key)
            .await
            .id
    }

    /// Secret de ce type rangé dans le trousseau pour ce compte, s'il y en a un
    pub fn stored_secret(&self, account: &AccountId, kind: SecretKind) -> Option<String> {
        let key = SecretRef {
            profile: ProfileId::from_raw("default"),
            account: account.clone(),
            kind,
        };
        let value = self.secrets.get(&key).unwrap();
        value.map(|secret| secret.expose().to_string())
    }
}

/// Serveurs en clair sur la boucle locale (le port SMTP n'est pas utilisé par ces tests)
pub fn local_servers(imap_port: u16) -> ServerSettings {
    let endpoint = |port: u16| ServerEndpoint {
        host: "127.0.0.1".to_string(),
        port,
        security: Security::Plaintext,
    };
    ServerSettings {
        imap: endpoint(imap_port),
        smtp: endpoint(1),
    }
}

/// Redirige les domaines de test vers le faux serveur IMAP
fn redirect_domains(config: &mut EngineConfig, imap_port: u16) {
    for domain in ["folders.test", "labels.test", "oauth.test"] {
        let servers = local_servers(imap_port);
        config
            .network
            .server_overrides
            .insert(domain.to_string(), servers);
    }
}

/// Démarre le faux serveur, déclare ses utilisateurs et ouvre un moteur qui le vise
pub async fn start() -> Fixture {
    let store = FakeMailStore::new();
    let credentials = Credentials::default();
    credentials.add_password(FOLDERS_USER, FOLDERS_PASSWORD);
    credentials.add_password(LABELS_USER, LABELS_PASSWORD);
    let imap = FakeImapServer::start(store.clone(), credentials.clone(), CapabilitySet::full())
        .await
        .unwrap();
    let oauth = FakeOAuthServer::start(credentials.clone()).await.unwrap();
    let home = tempfile::tempdir().unwrap();
    let secrets = Arc::new(MemorySecretStore::new());
    let mut config = config_with_secrets(home.path(), Arc::clone(&secrets));
    redirect_domains(&mut config, imap.port());
    super::oauth::point_oauth_at(&mut config, &oauth);
    let engine = Engine::open(config, CONTRACT_VERSION).unwrap();
    Fixture {
        home,
        engine,
        secrets,
        imap,
        store,
        oauth,
        credentials,
    }
}
