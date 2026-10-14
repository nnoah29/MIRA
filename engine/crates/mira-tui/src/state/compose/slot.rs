//! Emplacement de l'éditeur : fermé, en attente du brouillon du moteur, ou ouvert.

use mira_assembly::mira_vocabulary::IdempotencyKey;

use super::state::ComposeState;

/// État de la rédaction dans l'application.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ComposeSlot {
    #[default]
    Closed,
    Creating(IdempotencyKey),
    Open(Box<ComposeState>),
}

impl ComposeSlot {
    /// Vrai si aucun brouillon n'est ouvert ni demandé.
    pub fn is_closed(&self) -> bool {
        matches!(self, ComposeSlot::Closed)
    }

    /// Vrai tant que la création d'un brouillon attend la réponse du moteur.
    pub fn is_creating(&self) -> bool {
        matches!(self, ComposeSlot::Creating(_))
    }

    /// Vrai si la création attendue est celle de ce jeton.
    pub fn awaits(&self, token: &IdempotencyKey) -> bool {
        matches!(self, ComposeSlot::Creating(awaited) if awaited == token)
    }

    /// Brouillon ouvert, s'il y en a un.
    pub fn open(&self) -> Option<&ComposeState> {
        match self {
            ComposeSlot::Open(compose) => Some(compose),
            ComposeSlot::Closed | ComposeSlot::Creating(_) => None,
        }
    }

    /// Brouillon ouvert, modifiable.
    pub fn open_mut(&mut self) -> Option<&mut ComposeState> {
        match self {
            ComposeSlot::Open(compose) => Some(compose),
            ComposeSlot::Closed | ComposeSlot::Creating(_) => None,
        }
    }
}
