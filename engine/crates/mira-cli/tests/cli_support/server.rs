//! Banc d'essai : faux serveur IMAP du testkit, profil jetable et faux trousseau partagés entre commandes.

use std::sync::Arc;

use mira_assembly::accounts::NetworkPolicy;
use mira_assembly::mira_vocabulary::{
    AccountId, ProfileId, SecretKind, SecretRef, Security, ServerEndpoint, ServerSettings,
};
use mira_assembly::ports::{MemorySecretStore, SecretStore};
use mira_testkit::Credentials;
use mira_testkit::imap::{CapabilitySet, FakeImapServer, FakeMailStore};
use tempfile::TempDir;

use super::{CliResult, account_id_of, run_with, test_deps};

pub const USER: &str = "alice@folders.test";
pub const PASSWORD: &str = "pa55-SECRET-xyz";

/// Serveur IMAP local, profil jetable et trousseau en mémoire.
pub struct Bench {
    pub home: TempDir,
    pub secrets: Arc<MemorySecretStore>,
    pub imap: FakeImapServer,
}

/// Serveurs en clair sur la boucle locale (le SMTP n'est pas utilisé).
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

/// Démarre le faux serveur avec l'utilisateur de test.
pub async fn start_bench() -> Bench {
    let credentials = Credentials::default();
    credentials.add_password(USER, PASSWORD);
    let store = FakeMailStore::new();
    let imap = FakeImapServer::start(store, credentials, CapabilitySet::full())
        .await
        .unwrap();
    Bench {
        home: tempfile::tempdir().unwrap(),
        secrets: Arc::new(MemorySecretStore::new()),
        imap,
    }
}

impl Bench {
    /// Exécute une commande avec `stdin` ; le domaine `folders.test` vise le faux serveur.
    pub async fn run(&self, stdin: &str, arguments: &[&str]) -> CliResult {
        let mut network = NetworkPolicy::default();
        network
            .server_overrides
            .insert("folders.test".to_string(), local_servers(self.imap.port()));
        let mut deps = test_deps(self.home.path(), Arc::clone(&self.secrets), network, stdin);
        run_with(&mut deps, arguments).await
    }

    /// Mot de passe rangé au trousseau pour ce compte, s'il y en a un.
    pub fn stored_password(&self, account: &str) -> Option<String> {
        let key = SecretRef {
            profile: ProfileId::from_raw("default"),
            account: AccountId::from_raw(account),
            kind: SecretKind::Password,
        };
        let value = self.secrets.get(&key).unwrap();
        value.map(|secret| secret.expose().to_string())
    }

    /// Ajoute le compte de test (mot de passe sur stdin) et rend son identifiant.
    pub async fn add_default_account(&self) -> String {
        let stdin = format!("{PASSWORD}\n");
        let arguments = [
            "account",
            "add",
            "--address",
            USER,
            "--password-stdin",
            "--json",
        ];
        let added = self.run(&stdin, &arguments).await;
        account_id_of(&added)
    }
}
