//! Choix des stratégies d'après les capacités effectives (D-22) ; l'appelant journalise les replis.

use super::effective::EffectiveCapabilities;
use super::server_capability::ServerCapability;

/// Façon de détecter les changements d'une boîte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeDetection {
    Qresync,
    Condstore,
    FullCompare,
}

/// Façon de déplacer un message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveMethod {
    Move,
    CopyThenDelete,
}

/// Façon d'apprendre les nouveautés en temps réel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Realtime {
    Idle,
    Polling,
}

/// Façon de purger les messages supprimés.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpungeMethod {
    UidExpunge,
    NoPurge,
}

/// Choisit QRESYNC, sinon CONDSTORE, sinon la comparaison complète.
pub fn choose_change_detection(capabilities: &EffectiveCapabilities) -> ChangeDetection {
    if capabilities.has(ServerCapability::Qresync) {
        ChangeDetection::Qresync
    } else if capabilities.has(ServerCapability::Condstore) {
        ChangeDetection::Condstore
    } else {
        ChangeDetection::FullCompare
    }
}

/// Choisit MOVE, sinon copie puis suppression.
pub fn choose_move(capabilities: &EffectiveCapabilities) -> MoveMethod {
    if capabilities.has(ServerCapability::Move) {
        MoveMethod::Move
    } else {
        MoveMethod::CopyThenDelete
    }
}

/// Choisit IDLE, sinon l'interrogation périodique.
pub fn choose_realtime(capabilities: &EffectiveCapabilities) -> Realtime {
    if capabilities.has(ServerCapability::Idle) {
        Realtime::Idle
    } else {
        Realtime::Polling
    }
}

/// Choisit la purge ciblée par UID ; sans UIDPLUS on ne purge pas, par prudence (10.14).
pub fn choose_expunge(capabilities: &EffectiveCapabilities) -> ExpungeMethod {
    if capabilities.has(ServerCapability::UidPlus) {
        ExpungeMethod::UidExpunge
    } else {
        ExpungeMethod::NoPurge
    }
}
