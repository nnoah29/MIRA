//! État global du moteur (12.4, Moteur) : lecture locale uniquement.

use mira_vocabulary::MiraError;

use crate::api::contract::{AccountStatus, EngineStatus, Response};
use crate::api::engine::Engine;

/// Rend le profil, l'emplacement des données, les comptes et leurs actions en attente
pub(crate) fn handle_get_status(engine: &Engine) -> Result<Response, MiraError> {
    let accounts = engine.database.read(|tx| {
        tx.list_accounts()?
            .into_iter()
            .map(|account| {
                let pending_actions = tx.pending_action_count(&account.id)?;
                Ok(AccountStatus {
                    account,
                    pending_actions,
                })
            })
            .collect::<Result<Vec<_>, MiraError>>()
    })?;
    Ok(Response::Status(EngineStatus {
        profile: engine.config.profile.clone(),
        data_directory: engine.paths.root.display().to_string(),
        accounts,
    }))
}
