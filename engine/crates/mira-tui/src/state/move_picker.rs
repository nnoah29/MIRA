//! Choix d'une boîte de destination pour des messages.

use mira_assembly::mira_vocabulary::{Mailbox, MessageSummary};

/// Choix d'une boîte de destination pour des messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MovePicker {
    pub messages: Vec<MessageSummary>,
    targets: Vec<Mailbox>,
    selected: usize,
}

impl MovePicker {
    /// Sélecteur sur ces boîtes, la première choisie.
    pub fn new(messages: Vec<MessageSummary>, targets: Vec<Mailbox>) -> MovePicker {
        MovePicker {
            messages,
            targets,
            selected: 0,
        }
    }

    /// Boîtes proposées.
    pub fn targets(&self) -> &[Mailbox] {
        &self.targets
    }

    /// Index de la boîte choisie.
    pub fn selected(&self) -> usize {
        self.selected
    }

    /// Descend d'une boîte sans dépasser la dernière.
    pub fn down(&mut self) {
        self.selected = (self.selected + 1).min(self.targets.len().saturating_sub(1));
    }

    /// Monte d'une boîte sans dépasser la première.
    pub fn up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    /// Boîte actuellement choisie.
    pub fn chosen(&self) -> Option<&Mailbox> {
        self.targets.get(self.selected)
    }
}
