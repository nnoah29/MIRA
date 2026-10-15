//! Fiches affichées : les envois de la boîte d'envoi, puis les autres actions de la file.

use mira_assembly::api::OutboxEntry;
use mira_assembly::mira_vocabulary::{ActionId, ActionKind, ActionState, PendingAction};

use super::QueueState;

/// Ce que vise une fiche : un envoi (son objet) ou une autre action (sa nature et son nombre de messages).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueueItem {
    Send { subject: String },
    Action { kind: ActionKind, targets: usize },
}

/// Une ligne de la vue : action du moteur, ce qu'elle vise, son état, sa dernière erreur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueueEntry {
    pub action: ActionId,
    pub item: QueueItem,
    pub state: ActionState,
    pub last_error: Option<String>,
}

impl QueueState {
    /// Remplace les envois lus et recadre la sélection.
    pub fn replace_outbox(&mut self, outbox: Vec<OutboxEntry>) {
        self.outbox = outbox;
        self.rebuild();
    }

    /// Remplace les actions lues (les envois, déjà dans la boîte d'envoi, sont écartés) et recadre la sélection.
    pub fn replace_actions(&mut self, actions: Vec<PendingAction>) {
        self.actions = actions
            .into_iter()
            .filter(|action| !matches!(action.kind, ActionKind::Send { .. }))
            .collect();
        self.rebuild();
    }

    /// Fiches affichées, envois d'abord.
    pub fn entries(&self) -> &[QueueEntry] {
        &self.entries
    }

    /// Fiche sélectionnée.
    pub fn selected(&self) -> Option<&QueueEntry> {
        self.selection
            .selected
            .and_then(|index| self.entries.get(index))
    }

    /// Reconstruit les fiches depuis les deux lectures.
    fn rebuild(&mut self) {
        let sends = self.outbox.iter().map(|entry| QueueEntry {
            action: entry.action.clone(),
            item: QueueItem::Send {
                subject: entry.subject.clone(),
            },
            state: entry.state,
            last_error: entry.last_error.clone(),
        });
        let others = self.actions.iter().map(|action| QueueEntry {
            action: action.id.clone(),
            item: QueueItem::Action {
                kind: action.kind.clone(),
                targets: action.targets.len(),
            },
            state: action.state,
            last_error: action.last_error.clone(),
        });
        self.entries = sends.chain(others).collect();
        self.selection.clamp_to(self.entries.len());
    }
}
