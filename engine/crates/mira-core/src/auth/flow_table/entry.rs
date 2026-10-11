//! Un flux OAuth en cours dans le moteur, avec le compte qu'il a réservé.

use std::sync::Arc;

use mira_vocabulary::{AccountId, EmailAddress, ProviderId};
use tokio::sync::Notify;

use crate::auth::PendingFlow;

/// Flux enregistré : le flux lui-même tant qu'il attend, le signal d'annulation et la cible.
#[derive(Debug)]
pub struct FlowEntry {
    pub(super) pending: Option<PendingFlow>,
    pub(super) cancel: Arc<Notify>,
    pub(super) target: FlowTarget,
    pub(super) completed: bool,
}

/// Ce que le flux vise : compte réservé, adresse, provider, et réauthentification ou non.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowTarget {
    pub account: AccountId,
    pub address: EmailAddress,
    pub provider: ProviderId,
    pub reauthentication: bool,
}

/// État visible d'un flux : sa cible et s'il a déjà obtenu ses jetons.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowSummary {
    pub target: FlowTarget,
    pub completed: bool,
}

impl FlowEntry {
    /// Crée l'entrée d'un flux qui attend la redirection du navigateur
    pub fn new(pending: PendingFlow, target: FlowTarget) -> Self {
        Self {
            pending: Some(pending),
            cancel: Arc::new(Notify::new()),
            target,
            completed: false,
        }
    }

    /// Décrit l'entrée sans exposer le flux
    pub(super) fn summary(&self) -> FlowSummary {
        FlowSummary {
            target: self.target.clone(),
            completed: self.completed,
        }
    }
}
