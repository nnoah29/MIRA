//! Le moteur ouvert sur un profil : point d'entrée unique des clients (CLI, Flutter, agent).

use std::sync::Arc;
use std::time::Duration;

use mira_vocabulary::{ContractVersion, EventFilter, EventKind, MiraError};

use super::config::EngineConfig;
use super::contract::{Request, Response};
use super::router::route;
use super::startup::{
    check_client_version, open_profile, resume_interrupted_deletions, start_background,
};
use crate::auth::{CredentialVault, OAuthFlows};
use crate::events::{EventBus, EventSink, EventSubscription};
use crate::fetching::ServerSearchResults;
use crate::operations::OperationRegistry;
use crate::profile::{ProfileLock, ProfilePaths};
use crate::replayer::ReplayerHandle;
use crate::storage::Database;
use crate::sync::ConnectionBudgets;
use crate::sync_service::SyncService;

/// Moteur ouvert : un profil, sa base, son verrou, son bus d'événements.
#[derive(Debug)]
pub struct Engine {
    pub(crate) config: EngineConfig,
    pub(crate) paths: ProfilePaths,
    pub(crate) database: Arc<Database>,
    pub(crate) events: EventBus,
    pub(crate) operations: OperationRegistry,
    pub(crate) oauth_flows: OAuthFlows,
    pub(crate) server_searches: ServerSearchResults,
    pub(crate) account_lock: tokio::sync::Mutex<()>,
    pub(crate) replayers: ReplayerHandle,
    pub(crate) sync: SyncService,
    pub(crate) budgets: ConnectionBudgets,
    _lock: ProfileLock,
}

impl Engine {
    /// Ouvre le moteur sur un profil après avoir vérifié la version du client
    pub fn open(config: EngineConfig, client_version: ContractVersion) -> Result<Self, MiraError> {
        check_client_version(client_version)?;
        let (paths, lock, database) = open_profile(&config.profile, &config.environment)?;
        let events = EventBus::new(Arc::clone(&database), Arc::clone(&config.clock));
        let sink: Arc<dyn EventSink> = Arc::new(events.clone());
        let operations = OperationRegistry::new(Arc::clone(&config.ids), sink);
        events.emit(EventKind::EngineStarted)?;
        resume_interrupted_deletions(&config, &database, &events, &paths.attachments_cache);
        let budgets = ConnectionBudgets::default();
        let (replayers, sync) = start_background(&config, &database, &events, &budgets);
        Ok(Self {
            config,
            paths,
            database,
            events,
            operations,
            oauth_flows: OAuthFlows::new(),
            server_searches: ServerSearchResults::default(),
            account_lock: tokio::sync::Mutex::new(()),
            replayers,
            sync,
            budgets,
            _lock: lock,
        })
    }

    /// Exécute une requête du contrat
    pub async fn execute(&self, request: Request) -> Result<Response, MiraError> {
        route(self, request).await
    }

    /// Coffre d'identifiants au-dessus du trousseau et de la base du profil
    pub(crate) fn vault(&self) -> CredentialVault {
        CredentialVault::new(Arc::clone(&self.config.secrets), Arc::clone(&self.database))
    }

    /// S'abonne aux événements à partir d'un numéro déjà vu
    pub fn subscribe(&self, filter: EventFilter) -> EventSubscription {
        self.events.subscribe(filter)
    }

    /// Ferme le moteur sans attendre : les tâches de fond sont interrompues, le verrou du profil est libéré
    pub fn shutdown(self) -> Result<(), MiraError> {
        self.sync.abort_all();
        self.replayers.stop();
        self.events.emit(EventKind::EngineStopping)?;
        Ok(())
    }

    /// Ferme proprement le moteur : les synchronisations en cours ont `deadline` pour finir, puis sont interrompues ; rend `false` si le délai a été dépassé
    pub async fn shutdown_within(self, deadline: Duration) -> Result<bool, MiraError> {
        let on_time = self.sync.shutdown(deadline).await;
        self.shutdown()?;
        Ok(on_time)
    }
}
