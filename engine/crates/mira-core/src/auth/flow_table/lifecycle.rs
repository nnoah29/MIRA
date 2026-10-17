//! Étapes d'un flux dans la table : prise en charge, fin, annulation, abandon avec le compte.

use std::sync::Arc;

use mira_vocabulary::{AccountId, MiraError};
use tokio::sync::Notify;

use std::collections::BTreeMap;

use super::entry::{FlowEntry, FlowSummary};
use super::errors::unknown_flow;
use super::table::OAuthFlows;
use crate::auth::PendingFlow;

impl OAuthFlows {
    /// Sort le flux de la table pour l'attendre ; son signal d'annulation reste joignable
    pub fn begin_completion(
        &self,
        flow: &str,
    ) -> Result<(PendingFlow, Arc<Notify>, FlowSummary), MiraError> {
        let mut entries = self.entries();
        self.purge_expired(&mut entries);
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

    /// Décrit un flux qui a déjà obtenu ses jetons, ou échoue s'il est inconnu ou inachevé
    pub fn completed_summary(&self, flow: &str) -> Result<FlowSummary, MiraError> {
        let summary = self.entries().get(flow).map(FlowEntry::summary);
        match summary {
            Some(summary) if summary.completed => Ok(summary),
            _ => Err(unknown_flow()),
        }
    }

    /// Oublie les flux en attente depuis plus que la durée de vie et libère leur écouteur
    pub(super) fn purge_expired(&self, entries: &mut BTreeMap<String, FlowEntry>) {
        entries.retain(|_, entry| {
            let expired =
                entry.pending.is_some() && !entry.completed && entry.created.elapsed() >= self.ttl;
            if expired {
                entry.cancel.notify_one();
            }
            !expired
        });
    }
}
