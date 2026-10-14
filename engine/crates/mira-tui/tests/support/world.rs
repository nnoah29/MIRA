//! Monde de test : un moteur ouvert sur un profil jetable, éventuellement un compte et son faux serveur.

use std::sync::Arc;

use mira_assembly::accounts::NetworkPolicy;
use mira_assembly::api::{Engine, EngineConfig};
use mira_assembly::mira_vocabulary::AccountId;
use mira_testkit::imap::FakeImapServer;
use mira_tui::app::{TuiError, close_engine, open_engine};
use tempfile::TempDir;

use super::config::test_config;

/// Profil jetable et moteur ouvert dessus ; le faux serveur IMAP et le compte sont optionnels.
pub struct TestWorld {
    pub(super) home: TempDir,
    pub(super) config: EngineConfig,
    pub(super) engine: Arc<Engine>,
    pub(super) account: Option<AccountId>,
    pub(super) imap: Option<FakeImapServer>,
}

impl TestWorld {
    /// Monde vide : aucun compte, aucune synchronisation de fond.
    pub fn new() -> Self {
        let home = tempfile::tempdir().unwrap();
        Self::open_in(home, NetworkPolicy::default(), None)
    }

    /// Ouvre le moteur sur ce dossier avec ce réseau, et garde le serveur éventuel en vie.
    pub(super) fn open_in(
        home: TempDir,
        network: NetworkPolicy,
        imap: Option<FakeImapServer>,
    ) -> Self {
        let config = test_config(home.path(), network);
        let engine = open_engine(config.clone()).unwrap();
        Self {
            home,
            config,
            engine,
            account: None,
            imap,
        }
    }

    /// Poignée partagée sur le moteur ouvert.
    pub fn engine(&self) -> Arc<Engine> {
        Arc::clone(&self.engine)
    }

    /// Compte ajouté par `with_imap_account`, s'il y en a un.
    pub fn account(&self) -> Option<AccountId> {
        self.account.clone()
    }

    /// Ferme le moteur ; échoue tant qu'une poignée obtenue par `engine()` est encore vivante.
    pub async fn close(self) -> Result<(), TuiError> {
        close_engine(self.engine).await
    }
}
