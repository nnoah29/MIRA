//! Reprise au démarrage : les messages restés en attente d'évaluation après un arrêt brutal sont traités.

use std::sync::Arc;

use mira_vocabulary::{Account, MessageId, MiraError};

use super::submit_actions::apply_rules_with;
use crate::actions::QueueContext;
use crate::api::EngineConfig;
use crate::events::EventBus;
use crate::registry::resolve_account_provider;
use crate::storage::Database;

/// Rejoue l'évaluation de chaque message en attente ; un échec est journalisé et la ligne reste pour la fois suivante
pub fn replay_pending_rules(
    config: &EngineConfig,
    database: &Arc<Database>,
    events: &EventBus,
) -> Result<(), MiraError> {
    let queue = QueueContext {
        db: database,
        clock: config.clock.as_ref(),
        ids: config.ids.as_ref(),
        events,
    };
    for message in database.read(|tx| tx.list_pending())? {
        if let Err(error) = replay_one(config, &queue, &message) {
            tracing::warn!(code = %error.code, "règles non rejouées pour un message");
        }
    }
    Ok(())
}

/// Compte propriétaire du message en attente, s'il existe encore avec son compte
fn owner_of(queue: &QueueContext<'_>, message: &MessageId) -> Result<Option<Account>, MiraError> {
    queue.db.read(|tx| match tx.get_message(message)? {
        Some(summary) => tx.get_account(&summary.account),
        None => Ok(None),
    })
}

/// Rejoue un message : particularités du service de son compte, puis application des règles
fn replay_one(
    config: &EngineConfig,
    queue: &QueueContext<'_>,
    message: &MessageId,
) -> Result<(), MiraError> {
    let Some(account) = owner_of(queue, message)? else {
        return queue.db.write(|tx| tx.remove_pending(message));
    };
    let provider = resolve_account_provider(&config.registry, &account.provider)?;
    apply_rules_with(queue, &provider.descriptor().quirks, message)?;
    Ok(())
}
