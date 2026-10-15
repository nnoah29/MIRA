//! Modifications de la fiche sélectionnée : réessayer, annuler, ouvrir la décision sur un envoi incertain.

use mira_assembly::api::{ComposeRequest, QueueRequest, Request};
use mira_assembly::mira_vocabulary::{ActionId, ActionState};

use crate::commands::{Command, Tag};
use crate::state::queue::{QueueEntry, QueueItem};
use crate::state::{AppState, Popup};
use crate::view::safe_text::display_safe;

/// Commande de modification de la file qui vise cette action.
pub fn change(action: ActionId, request: Request) -> Vec<Command> {
    vec![Command::Execute {
        tag: Tag::QueueChange(action),
        request,
    }]
}

/// `r` : relance une action échouée (un envoi garde sa clé et son action : pas de doublon).
pub fn retry(state: &mut AppState) -> Vec<Command> {
    let Some(entry) = selected_entry(state) else {
        return Vec::new();
    };
    if entry.state != ActionState::Failed {
        return say(state, "Seule une action échouée peut être réessayée");
    }
    state.status.message = Some("Nouvel essai demandé".to_string());
    let action = entry.action.clone();
    change(
        entry.action,
        Request::Queue(QueueRequest::RetryAction { action }),
    )
}

/// `x` : annule un envoi ou une action encore en attente.
pub fn cancel(state: &mut AppState) -> Vec<Command> {
    let Some(entry) = selected_entry(state) else {
        return Vec::new();
    };
    if entry.state != ActionState::Pending {
        return say(state, "Seule une action en attente peut être annulée");
    }
    let action = entry.action.clone();
    let (request, done) = match entry.item {
        QueueItem::Send { subject } => {
            let done = format!("Annulation demandée pour « {} »", display_safe(&subject));
            state.queue.note_cancel(action.clone(), subject);
            (
                Request::Compose(ComposeRequest::CancelSend { action }),
                done,
            )
        }
        QueueItem::Action { .. } => (
            Request::Queue(QueueRequest::CancelAction { action }),
            "Action annulée".to_string(),
        ),
    };
    state.status.message = Some(done);
    change(entry.action, request)
}

/// `u` ou Entrée : ouvre la décision « renvoyer ou non » d'un envoi incertain ; jamais de renvoi sans elle.
pub fn resolve(state: &mut AppState) -> Vec<Command> {
    let Some(entry) = selected_entry(state) else {
        return Vec::new();
    };
    let QueueEntry {
        action,
        item: QueueItem::Send { subject },
        state: ActionState::Uncertain,
        ..
    } = entry
    else {
        return say(state, "Cette action n'attend aucune décision");
    };
    state.popup = Popup::ResolveUncertain { action, subject };
    Vec::new()
}

/// Fiche visée par la touche ; si l'action sélectionnée a disparu, le dit et ne vise rien d'autre.
fn selected_entry(state: &mut AppState) -> Option<QueueEntry> {
    let entry = state.queue.selected().cloned();
    if entry.is_none() && !state.queue.entries().is_empty() {
        state.status.message = Some("Cette action n'existe plus".to_string());
    }
    entry
}

/// Montre ce message dans la barre d'état, sans rien envoyer.
fn say(state: &mut AppState, text: &str) -> Vec<Command> {
    state.status.message = Some(text.to_string());
    Vec::new()
}
