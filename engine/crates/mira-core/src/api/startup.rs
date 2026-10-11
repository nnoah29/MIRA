//! Étapes de l'ouverture : version du contrat, emplacements, verrou, base.

use std::sync::Arc;

use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{CONTRACT_VERSION, ContractVersion, ErrorCategory, MiraError, ProfileId};

use crate::profile::{ProfileEnvironment, ProfileLock, ProfilePaths};
use crate::storage::Database;

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
