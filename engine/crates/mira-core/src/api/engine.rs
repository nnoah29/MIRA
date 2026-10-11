//! Le moteur ouvert sur un profil : point d'entrée unique des clients (CLI, Flutter, agent).

use std::sync::Arc;

use mira_vocabulary::{ContractVersion, EventFilter, EventKind, MiraError};

use super::config::EngineConfig;
use super::contract::{Request, Response};
use super::router::route;
use super::startup::{check_client_version, open_profile};
use crate::events::{EventBus, EventSink, EventSubscription};
use crate::operations::OperationRegistry;
use crate::profile::{ProfileLock, ProfilePaths};
use crate::storage::Database;

/// Moteur ouvert : un profil, sa base, son verrou, son bus d'événements.
#[derive(Debug)]
pub struct Engine {
    pub(crate) config: EngineConfig,
    pub(crate) paths: ProfilePaths,
    pub(crate) database: Arc<Database>,
    pub(crate) events: EventBus,
    pub(crate) operations: OperationRegistry,
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
        Ok(Self {
            config,
            paths,
            database,
            events,
            operations,
            _lock: lock,
        })
    }

    /// Exécute une requête du contrat
    pub async fn execute(&self, request: Request) -> Result<Response, MiraError> {
        route(self, request).await
    }

    /// S'abonne aux événements à partir d'un numéro déjà vu
    pub fn subscribe(&self, filter: EventFilter) -> EventSubscription {
        self.events.subscribe(filter)
    }

    /// Ferme proprement le moteur ; le verrou du profil est libéré
    pub fn shutdown(self) -> Result<(), MiraError> {
        self.events.emit(EventKind::EngineStopping)?;
        Ok(())
    }
}
