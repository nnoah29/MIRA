//! Services que les handlers demandent au moteur : accès réseau d'un compte, contexte de tâche longue.

use std::sync::Arc;

use mira_vocabulary::{AccountId, MiraError, Provider};

use super::engine::Engine;
use crate::events::EventSink;
use crate::fetching::{FetchTask, ImapAccess};
use crate::operations::OperationHandle;
use crate::sync::ConnectionBudget;

impl Engine {
    /// Accès IMAP aux comptes du profil, selon la politique réseau et le trousseau du moteur
    pub(crate) fn imap_access(&self) -> ImapAccess {
        ImapAccess::new(
            self.config.clone(),
            Arc::clone(&self.database),
            self.budgets.clone(),
        )
    }

    /// Budget de connexions de la clé (compte, ou adresse d'un compte pas encore ajouté), selon la limite du provider
    pub(crate) fn budget_for(&self, key: &str, provider: &dyn Provider) -> ConnectionBudget {
        let limit = provider.descriptor().limits.max_connections;
        self.budgets.budget_for(key, limit)
    }

    /// Contexte transportable d'une tâche longue, lié à la poignée de son opération
    pub(crate) fn fetch_task(&self, handle: OperationHandle) -> FetchTask {
        let events: Arc<dyn EventSink> = Arc::new(self.events.clone());
        FetchTask {
            access: self.imap_access(),
            database: Arc::clone(&self.database),
            ids: Arc::clone(&self.config.ids),
            events,
            handle,
            cache_directory: self.paths.attachments_cache.clone(),
        }
    }

    /// Identifiants de tous les comptes du profil
    pub(crate) fn account_ids(&self) -> Result<Vec<AccountId>, MiraError> {
        let accounts = self.database.read(|tx| tx.list_accounts())?;
        Ok(accounts.into_iter().map(|account| account.id).collect())
    }
}
