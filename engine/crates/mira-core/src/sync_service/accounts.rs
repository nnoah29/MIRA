//! Tâches de synchronisation par compte : démarrage au lancement du moteur, à l'ajout, à la reprise ; arrêt.

use mira_vocabulary::{Account, AccountId, AccountState, MiraError};

use super::service::{ActiveSync, SyncService};
use crate::registry::resolve_account_provider;
use crate::sync::SyncContext;

/// Contexte de synchronisation d'un compte : base, provider, horloge, identifiants, événements
pub(super) fn context_for(
    active: &ActiveSync,
    account: &Account,
) -> Result<SyncContext, MiraError> {
    let config = &active.config;
    Ok(SyncContext {
        db: active.database.clone(),
        provider: resolve_account_provider(&config.registry, &account.provider)?,
        account: account.clone(),
        clock: config.clock.clone(),
        ids: config.ids.clone(),
        events: active.events.clone(),
        network: config.network.clone(),
    })
}

impl SyncService {
    /// Lance la tâche du compte, sauf s'il est en pause ; rend `false` si rien n'a été lancé
    pub(crate) fn start_account(&self, account: &Account) -> Result<bool, MiraError> {
        let Some(active) = &self.active else {
            return Ok(false);
        };
        if account.state == AccountState::Paused {
            return Ok(false);
        }
        let ctx = context_for(active, account)?;
        Ok(active
            .supervisor
            .start_account(ctx, active.credentials.clone()))
    }

    /// Arrête la tâche du compte et attend qu'elle ait cessé d'écrire
    pub(crate) async fn stop_account(&self, account: &AccountId) {
        if let Some(active) = &self.active {
            active.supervisor.stop_account(account).await;
        }
    }

    /// Lance les tâches des comptes du profil qui ne sont ni en pause ni en cours de suppression
    pub(super) fn start_known_accounts(&self) {
        let Some(active) = &self.active else {
            return;
        };
        let known = active.database.read(|tx| {
            let mut kept = Vec::new();
            for account in tx.list_accounts()? {
                if !tx.account_is_deleting(&account.id)? {
                    kept.push(account);
                }
            }
            Ok(kept)
        });
        for account in known.unwrap_or_default() {
            if let Err(error) = self.start_account(&account) {
                tracing::warn!(code = %error.code, "tâche de synchronisation non lancée");
            }
        }
    }
}
