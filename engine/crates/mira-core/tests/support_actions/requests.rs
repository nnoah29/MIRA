//! Fabriques de demandes de soumission et raccourcis de soumission pour les tests.

use mira_core::actions::{SubmitRequest, submit};
use mira_vocabulary::{AccountId, ActionId, ActionKind, IdempotencyKey, MessageId};

use super::quirks::folders_quirks;
use super::world::World;

/// Demande sur le compte acc-1 visant les messages donnés
pub fn request(kind: ActionKind, targets: &[&str], key: &str) -> SubmitRequest {
    SubmitRequest {
        account: AccountId::from_raw("acc-1"),
        kind,
        targets: targets.iter().map(|id| MessageId::from_raw(*id)).collect(),
        key: IdempotencyKey(key.to_string()),
    }
}

/// Soumet sur un service à dossiers et rend l'identifiant de l'action
pub fn submit_folders(world: &World, kind: ActionKind, targets: &[&str], key: &str) -> ActionId {
    submit(&world.ctx(), &folders_quirks(), request(kind, targets, key)).unwrap()
}
