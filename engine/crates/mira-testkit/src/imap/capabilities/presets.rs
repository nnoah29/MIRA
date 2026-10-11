//! Ensembles d'extensions prêts à l'emploi.

use super::{Capability, CapabilitySet};

impl CapabilitySet {
    /// Toutes les extensions sauf celles propres à Gmail
    pub fn full() -> Self {
        Self::minimal()
            .with(Capability::Idle)
            .with(Capability::Condstore)
            .with(Capability::Qresync)
            .with(Capability::Move)
            .with(Capability::UidPlus)
            .with(Capability::SpecialUse)
            .with(Capability::Enable)
            .with(Capability::Id)
            .with(Capability::Literalplus)
            .with(Capability::Esearch)
    }

    /// Aucune extension : IMAP4rev1 seul
    pub fn minimal() -> Self {
        Self::default()
    }

    /// Toutes les extensions sauf QRESYNC
    pub fn without_qresync() -> Self {
        Self::full().without(Capability::Qresync)
    }

    /// Toutes les extensions sauf MOVE
    pub fn without_move() -> Self {
        Self::full().without(Capability::Move)
    }

    /// Profil proche de Gmail : X-GM-EXT-1, sans QRESYNC
    pub fn gmail() -> Self {
        Self::minimal()
            .with(Capability::Idle)
            .with(Capability::Condstore)
            .with(Capability::Move)
            .with(Capability::UidPlus)
            .with(Capability::SpecialUse)
            .with(Capability::Enable)
            .with(Capability::Id)
            .with(Capability::Gmail)
    }
}
