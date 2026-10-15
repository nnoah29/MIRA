//! Journal d'événements : page affichée, pile des pages déjà vues et lecture attendue.

use mira_assembly::mira_vocabulary::{Event, EventSeq};

use crate::state::Selection;

/// Nombre d'événements demandés par page.
pub const EVENTS_PAGE: u32 = 500;

/// Sens d'un changement de page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageMove {
    First,
    Next,
    Previous,
}

/// Lecture de page attendue : son jeton, le numéro après lequel elle lit et ce qu'elle doit faire de la pile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingPage {
    pub token: u64,
    pub after: Option<EventSeq>,
    pub movement: PageMove,
    pub seeking_end: bool,
}

/// Une page du journal ; on avance en lisant « après le dernier numéro » de la page affichée.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EventsState {
    pub(super) entries: Vec<Event>,
    pub selection: Selection,
    pub(super) after: Option<EventSeq>,
    pub(super) history: Vec<Option<EventSeq>>,
    pub(super) loaded: bool,
    pub(super) at_end: bool,
    pub(super) pending: Option<PendingPage>,
}

impl EventsState {
    /// Événements de la page affichée, du plus ancien au plus récent.
    pub fn entries(&self) -> &[Event] {
        &self.entries
    }

    /// Déplace la sélection dans la page.
    pub fn move_selection(&mut self, step: fn(&mut Selection, usize)) {
        step(&mut self.selection, self.entries.len());
    }

    /// Vrai une fois une page reçue.
    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    /// Vrai si la page affichée est la dernière du journal.
    pub fn at_end(&self) -> bool {
        self.at_end
    }

    /// Numéro de la page affichée (la première est la 1).
    pub fn page_number(&self) -> usize {
        self.history.len() + 1
    }
}
