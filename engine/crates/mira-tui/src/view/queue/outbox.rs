//! Texte d'une fiche : état, envoi (objet neutralisé) ou autre action, dernière erreur neutralisée.

use mira_assembly::mira_vocabulary::ActionState;

use super::actions::action_label;
use crate::state::queue::{QueueEntry, QueueItem};
use crate::view::safe_text::display_safe;

const TEXT_CHARS: usize = 200;

/// Nom affiché de l'état d'une action.
pub fn state_label(state: ActionState) -> &'static str {
    match state {
        ActionState::Pending => "en attente",
        ActionState::InProgress => "en cours",
        ActionState::Succeeded => "réussie",
        ActionState::Failed => "échouée",
        ActionState::Cancelled => "annulée",
        ActionState::Uncertain => "incertaine",
    }
}

/// Ligne de la fiche : état, ce qu'elle vise, puis l'erreur s'il y en a une.
pub fn entry_text(entry: &QueueEntry) -> String {
    let what = match &entry.item {
        QueueItem::Send { subject } => format!("Envoi « {} »", shown(subject)),
        QueueItem::Action { kind, targets } => action_label(kind, *targets),
    };
    let mut text = format!("{:<11} {what}", state_label(entry.state));
    if let Some(error) = &entry.last_error {
        text.push_str(" — ");
        text.push_str(&shown(error));
    }
    text
}

/// Texte venu du courrier ou du serveur : raccourci, puis neutralisé.
pub fn shown(text: &str) -> String {
    let cut: String = text.chars().take(TEXT_CHARS).collect();
    display_safe(&cut)
}
