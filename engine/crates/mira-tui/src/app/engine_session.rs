//! Ouverture et fermeture du moteur pour la durée d'une session plein écran.

use std::sync::Arc;
use std::time::Duration;

use mira_assembly::api::{Engine, EngineConfig};
use mira_assembly::mira_vocabulary::CONTRACT_VERSION;

use super::error::TuiError;

const SHUTDOWN_DEADLINE: Duration = Duration::from_secs(5);

/// Ouvre le moteur du profil à la version du contrat compilée, synchro de fond selon la configuration.
pub fn open_engine(config: EngineConfig) -> Result<Arc<Engine>, TuiError> {
    let engine = Engine::open(config, CONTRACT_VERSION)?;
    Ok(Arc::new(engine))
}

/// Ferme le moteur en laissant cinq secondes aux synchronisations ; échoue s'il est encore partagé ou si le délai est dépassé.
pub async fn close_engine(engine: Arc<Engine>) -> Result<(), TuiError> {
    let owned = Arc::try_unwrap(engine).map_err(|_| TuiError::EngineStillShared)?;
    match owned.shutdown_within(SHUTDOWN_DEADLINE).await? {
        true => Ok(()),
        false => Err(TuiError::ShutdownTimedOut),
    }
}
