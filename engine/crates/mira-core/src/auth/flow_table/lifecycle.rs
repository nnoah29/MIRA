//! Étapes d'un flux dans la table : prise en charge, fin, annulation, abandon avec le compte.

use std::sync::Arc;

use mira_vocabulary::{AccountId, MiraError};
use tokio::sync::Notify;

use super::entry::FlowSummary;
use super::table::{OAuthFlows, unknown_flow};
use crate::auth::PendingFlow;

impl OAuthFlows {
    /// Sort le flux de la table pour l'attendre ; son signal d'annulation reste joignable
    pub fn begin_completion(
        &self,
        flow: &str,
    ) -> Result<(PendingFlow, Arc<Notify>, FlowSummary), MiraError> {
        let mut entries = self.entries();
        let entry = entries.get_mut(flow).ok_or_else(unknown_flow)?;
        let pending = entry.pending.take().ok_or_else(unknown_flow)?;
        Ok((pending, Arc::clone(&entry.cancel), entry.summary()))
    }

    /// Marque le flux terminé ; faux si une annulation l'a retiré entre-temps
    pub fn mark_completed(&self, flow: &str) -> bool {
        match self.entries().get_mut(flow) {
            Some(entry) => {
                entry.completed = true;
                true
            }
            None => false,
        }
    }

    /// Annule et oublie le flux ; rend son état s'il existait (annulation répétée sans effet)
    pub fn cancel(&self, flow: &str) -> Option<FlowSummary> {
        let entry = self.entries().remove(flow)?;
        entry.cancel.notify_one();
        Some(entry.summary())
    }

    /// Annule tous les flux d'un compte qui disparaît
    pub fn discard_account(&self, account: &AccountId) {
        let mut entries = self.entries();
        entries.retain(|_, entry| {
            let same = entry.target.account == *account;
            if same {
                entry.cancel.notify_one();
            }
            !same
        });
    }
}
