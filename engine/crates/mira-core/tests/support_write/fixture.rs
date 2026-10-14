//! Banc complet : moteur ouvert, faux serveurs IMAP et SMTP, horloge manuelle, miroir garni.

use std::sync::Arc;

use mira_core::api::{Engine, EngineConfig};
use mira_core::ports::{ManualClock, MemorySecretStore};
use mira_core::profile::ProfilePaths;
use mira_core::storage::Database;
use mira_testkit::Credentials;
use mira_testkit::imap::{CapabilitySet, FakeImapServer, FakeMailStore};
use mira_testkit::smtp::FakeSmtpServer;
use mira_vocabulary::{AccountId, AuthMethodKind, CONTRACT_VERSION, Timestamp};
use tempfile::TempDir;

use super::build::{build_config, servers};
use super::seed::seed_mirror;
use super::servers::start_servers;
use super::{PASSWORD, START_MILLIS, USER};
use crate::support_api::requests::{add_account, draft};

/// Tout ce qu'un test d'écriture manipule.
pub struct Fixture {
    pub home: TempDir,
    pub secrets: Arc<MemorySecretStore>,
    pub clock: Arc<ManualClock>,
    pub store: FakeMailStore,
    pub credentials: Credentials,
    pub imap: FakeImapServer,
    pub smtp: FakeSmtpServer,
    pub config: EngineConfig,
    pub account: AccountId,
    pub db: Database,
    pub engine_slot: Option<Engine>,
}

impl Fixture {
    /// Démarre les faux serveurs, ouvre le moteur, ajoute le compte et garnit son miroir
    pub async fn start() -> Self {
        let up = start_servers().await;
        let home = tempfile::tempdir().unwrap();
        let secrets = Arc::new(MemorySecretStore::new());
        let clock = Arc::new(ManualClock::new(Timestamp::from_millis(START_MILLIS)));
        let wiring = servers(up.imap.port(), up.smtp.port());
        let config = build_config(home.path(), &secrets, &clock, "folders.test", wiring);
        let engine = Engine::open(config.clone(), CONTRACT_VERSION).unwrap();
        let request = draft(USER, AuthMethodKind::Password);
        let account = add_account(&engine, request, PASSWORD, "k0").await.id;
        let paths = ProfilePaths::resolve(&config.profile, &config.environment).unwrap();
        let db = Database::open(&paths.database).unwrap();
        seed_mirror(&db, &account, &up.store, USER);
        Self {
            home,
            secrets,
            clock,
            store: up.store,
            credentials: up.credentials,
            imap: up.imap,
            smtp: up.smtp,
            config,
            account,
            db,
            engine_slot: Some(engine),
        }
    }

    /// Le moteur ouvert
    pub fn engine(&self) -> &Engine {
        self.engine_slot.as_ref().unwrap()
    }

    /// Arrête le moteur puis en ouvre un autre sur le même profil (redémarrage)
    pub fn restart(&mut self) {
        if let Some(old) = self.engine_slot.take() {
            old.shutdown().unwrap();
        }
        let reopened = Engine::open(self.config.clone(), CONTRACT_VERSION).unwrap();
        self.engine_slot = Some(reopened);
    }

    /// Fait avancer l'horloge du moteur
    pub fn advance(&self, millis: i64) {
        self.clock.advance_millis(millis);
    }

    /// Coupe le serveur IMAP : plus aucune connexion n'aboutit
    pub async fn go_offline(&self) {
        self.imap.shutdown();
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    /// Relance le serveur IMAP au même port, avec le même courrier
    pub async fn go_online(&mut self) {
        let port = self.imap.port();
        let (store, credentials) = (self.store.clone(), self.credentials.clone());
        let caps = CapabilitySet::full();
        let restarted = FakeImapServer::start_on_port(port, store, credentials, caps);
        self.imap = restarted.await.unwrap();
    }
}
