//! Moteur de test relié à un faux serveur IMAP : les domaines de test sont redirigés vers lui.

use std::sync::Arc;

use mira_core::api::{Engine, EngineConfig};
use mira_core::ports::{MemorySecretStore, SecretStore, SystemClock};
use mira_core::sync::BackgroundSync;
use mira_testkit::Credentials;
use mira_testkit::imap::{CapabilitySet, FakeImapServer, FakeMailStore};
use mira_testkit::oauth::FakeOAuthServer;
use mira_testkit::smtp::FakeSmtpServer;
use mira_vocabulary::{
    AccountId, AuthMethodKind, CONTRACT_VERSION, ProfileId, SecretKind, SecretRef,
};
use tempfile::TempDir;

use super::config_with_secrets;
use super::endpoints::redirect_domains;
use super::requests::{add_account, draft};

pub const FOLDERS_USER: &str = "alice@folders.test";
pub const FOLDERS_PASSWORD: &str = "pa55-SECRET-xyz";
pub const LABELS_USER: &str = "bob@labels.test";
pub const LABELS_PASSWORD: &str = "app-SECRET-789";

/// Moteur ouvert, trousseau et faux serveur IMAP où `folders.test` et `labels.test` sont redirigés.
pub struct Fixture {
    pub home: TempDir,
    pub engine: Engine,
    pub config: EngineConfig,
    pub secrets: Arc<MemorySecretStore>,
    pub imap: FakeImapServer,
    pub smtp: FakeSmtpServer,
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

/// Règle la synchro de fond ; l'horloge est réelle quand elle est active
fn apply_sync_mode(config: &mut EngineConfig, sync: BackgroundSync) {
    if matches!(sync, BackgroundSync::On(_)) {
        config.clock = Arc::new(SystemClock);
    }
    config.network.background_sync = sync;
}

/// Démarre le faux serveur et un moteur sans synchronisation en arrière-plan
pub async fn start() -> Fixture {
    start_with(BackgroundSync::Off).await
}

/// Démarre le faux serveur, déclare ses utilisateurs et ouvre un moteur qui le vise ; l'horloge est réelle si la synchro est active
pub async fn start_with(sync: BackgroundSync) -> Fixture {
    let store = FakeMailStore::new();
    let credentials = Credentials::default();
    credentials.add_password(FOLDERS_USER, FOLDERS_PASSWORD);
    credentials.add_password(LABELS_USER, LABELS_PASSWORD);
    let imap = FakeImapServer::start(store.clone(), credentials.clone(), CapabilitySet::full())
        .await
        .unwrap();
    let smtp = FakeSmtpServer::start(credentials.clone()).await.unwrap();
    let oauth = FakeOAuthServer::start(credentials.clone()).await.unwrap();
    let home = tempfile::tempdir().unwrap();
    let secrets = Arc::new(MemorySecretStore::new());
    let mut config = config_with_secrets(home.path(), Arc::clone(&secrets));
    redirect_domains(&mut config, imap.port(), smtp.port());
    apply_sync_mode(&mut config, sync);
    super::oauth::point_oauth_at(&mut config, &oauth);
    let engine = Engine::open(config.clone(), CONTRACT_VERSION).unwrap();
    Fixture {
        home,
        engine,
        config,
        secrets,
        imap,
        smtp,
        store,
        oauth,
        credentials,
    }
}
