//! Étapes de l'ouverture : version du contrat, emplacements, verrou, base.

use std::path::Path;
use std::sync::Arc;

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{CONTRACT_VERSION, ContractVersion, ErrorCategory, MiraError, ProfileId};

use super::config::EngineConfig;
use crate::accounts::finish_interrupted_deletions;
use crate::auth::CredentialVault;
use crate::events::EventBus;
use crate::profile::{ProfileEnvironment, ProfileLock, ProfilePaths};
use crate::replayer::ReplayerHandle;
use crate::storage::Database;
use crate::sync::ConnectionBudgets;
use crate::sync_service::SyncService;

/// Refuse un client dont la version du contrat est incompatible (invariant 33)
pub(crate) fn check_client_version(client: ContractVersion) -> Result<(), MiraError> {
    if CONTRACT_VERSION.is_compatible_with(client) {
        return Ok(());
    }
    Err(MiraError::new(
        ErrorCategory::InvalidInput,
        invalid_input::CONTRACT_VERSION,
        format!(
            "contrat client {}.{} incompatible avec le moteur {}.{}",
            client.major, client.minor, CONTRACT_VERSION.major, CONTRACT_VERSION.minor
        ),
    ))
}

/// Résout les emplacements du profil, prend le verrou exclusif (D-27) et ouvre la base
pub(crate) fn open_profile(
    profile: &ProfileId,
    environment: &ProfileEnvironment,
) -> Result<(ProfilePaths, ProfileLock, Arc<Database>), MiraError> {
    let paths = ProfilePaths::resolve(profile, environment)?;
    let lock = ProfileLock::acquire(&paths)?;
    let database = Arc::new(Database::open(&paths.database)?);
    Ok((paths, lock, database))
}

/// Achève les suppressions de compte interrompues par un arrêt brutal (S14) ; un échec est journalisé sans empêcher l'ouverture
pub(crate) fn resume_interrupted_deletions(
    config: &EngineConfig,
    database: &Arc<Database>,
    events: &EventBus,
    cache_directory: &Path,
) {
    let vault = CredentialVault::new(Arc::clone(&config.secrets), Arc::clone(database));
    if let Err(error) = finish_interrupted_deletions(database, &vault, events, cache_directory) {
        tracing::warn!(code = %error.code, "suppression de compte interrompue non achevée");
    }
}

/// Lance le rejeu et la synchronisation de fond, qui partagent les budgets de connexions des comptes
pub(crate) fn start_background(
    config: &EngineConfig,
    database: &Arc<Database>,
    events: &EventBus,
    budgets: &ConnectionBudgets,
) -> (ReplayerHandle, SyncService) {
    let replayers = ReplayerHandle::start(
        config.clone(),
        Arc::clone(database),
        events.clone(),
        budgets.clone(),
    );
    (
        replayers,
        SyncService::start(config, database, events, budgets),
    )
}
