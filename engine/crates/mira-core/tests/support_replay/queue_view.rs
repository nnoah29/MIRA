//! Lectures de contrôle de la file et des événements pour les assertions du rejeu.

use mira_core::actions::SubmitRequest;
use mira_vocabulary::{ActionId, ActionKind, ActionState, EventKind};

use super::fixture::Fixture;
use crate::support_actions::inspect::action;
use crate::support_actions::requests::request;

/// État d'une action relu en base
pub fn state_of(fx: &Fixture, id: &ActionId) -> ActionState {
    action(&fx.world.db, id).state
}

/// Demande sur acc-1 visant les messages donnés
pub fn demand(kind: ActionKind, targets: &[&str], key: &str) -> SubmitRequest {
    request(kind, targets, key)
}

/// Nombre d'événements `ActionApplied` émis
pub fn applied_count(fx: &Fixture) -> usize {
    let kinds = fx.world.sink.kinds();
    let applied = |kind: &&EventKind| matches!(kind, EventKind::ActionApplied { .. });
    kinds.iter().filter(applied).count()
}
