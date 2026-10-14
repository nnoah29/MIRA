//! Monde des scénarios S2 à S7 : faux serveurs, profil jetable, coupure et retour du réseau.

use std::sync::{Arc, Mutex};

use mira_assembly::mira_vocabulary::{Security, ServerEndpoint, ServerSettings, Timestamp};
use mira_assembly::ports::{ManualClock, MemorySecretStore, SequentialIds};
use mira_cli::CliDeps;
use mira_testkit::Credentials;
use mira_testkit::imap::{CapabilitySet, FakeImapServer, FakeMailStore};
use mira_testkit::smtp::FakeSmtpServer;

use super::full_bench::{FullBench, START_MILLIS};

/// Faux serveurs d'un utilisateur, profil du CLI et journal de tout ce que les commandes ont écrit.
pub struct SaWorld {
    pub bench: FullBench,
    pub user: String,
    pub password: String,
    pub domain: String,
    pub imap_port: u16,
    pub transcript: Mutex<Vec<String>>,
}

/// Port local sur lequel personne n'écoute : toute connexion y est refusée.
pub fn closed_port() -> u16 {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

/// Point d'accès en clair sur la boucle locale.
fn plain_endpoint(port: u16) -> ServerEndpoint {
    ServerEndpoint {
        host: "127.0.0.1".to_string(),
        port,
        security: Security::Plaintext,
    }
}

impl SaWorld {
    /// Démarre les faux serveurs IMAP et SMTP pour cet utilisateur et ce mot de passe.
    pub async fn start(user: &str, password: &str, capabilities: CapabilitySet) -> Self {
        let credentials = Credentials::default();
        credentials.add_password(user, password);
        let store = FakeMailStore::new();
        let imap = FakeImapServer::start(store.clone(), credentials.clone(), capabilities)
            .await
            .unwrap();
        let smtp = FakeSmtpServer::start(credentials).await.unwrap();
        let imap_port = imap.port();
        let bench = FullBench {
            home: tempfile::tempdir().unwrap(),
            secrets: Arc::new(MemorySecretStore::new()),
            clock: Arc::new(ManualClock::new(Timestamp::from_millis(START_MILLIS))),
            ids: Arc::new(SequentialIds::new()),
            store,
            imap,
            smtp,
        };
        let domain = user.rsplit('@').next().unwrap().to_string();
        let transcript = Mutex::new(Vec::new());
        let (user, password) = (user.to_string(), password.to_string());
        Self {
            bench,
            user,
            password,
            domain,
            imap_port,
            transcript,
        }
    }

    /// Dépendances du CLI : le domaine de l'utilisateur vise les faux serveurs (ou un port fermé hors ligne).
    pub fn deps(&self, stdin: &str) -> CliDeps {
        let mut deps = self.bench.deps(stdin);
        let servers = ServerSettings {
            imap: plain_endpoint(self.imap_port),
            smtp: plain_endpoint(self.bench.smtp.port()),
        };
        if !self.domain.is_empty() {
            deps.network
                .server_overrides
                .insert(self.domain.clone(), servers);
        }
        deps
    }

    /// Coupe le réseau : les commandes suivantes visent un port où rien n'écoute.
    pub fn go_offline(&mut self) {
        self.imap_port = closed_port();
    }

    /// Rétablit le réseau : les commandes suivantes visent à nouveau le faux serveur.
    pub fn go_online(&mut self) {
        self.imap_port = self.bench.imap.port();
    }
}
