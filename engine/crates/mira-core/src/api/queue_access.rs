//! Accès du moteur à la file d'actions : contexte local, particularités du service d'un compte, réveil du rejeu.

use mira_vocabulary::{AccountId, MiraError, ProviderQuirks};

use super::engine::Engine;
use crate::accounts::require_active_account;
use crate::actions::QueueContext;
use crate::registry::resolve_account_provider;

impl Engine {
    /// Contexte des opérations locales de la file (base, horloge, identifiants, événements)
    pub(crate) fn queue_context(&self) -> QueueContext<'_> {
        QueueContext {
            db: &self.database,
            clock: self.config.clock.as_ref(),
            ids: self.config.ids.as_ref(),
            events: &self.events,
        }
    }

    /// Particularités du service d'un compte actif, pour résoudre l'effet local des actions
    pub(crate) fn quirks_of(&self, account: &AccountId) -> Result<ProviderQuirks, MiraError> {
        let found = self
            .database
            .read(|tx| require_active_account(tx, account))?;
        let provider = resolve_account_provider(&self.config.registry, &found.provider)?;
        Ok(provider.descriptor().quirks)
    }

    /// Réveille le rejeu : une action vient d'être mise en file ou relancée
    pub(crate) fn wake_replayers(&self) {
        self.replayers.wake();
    }
}
