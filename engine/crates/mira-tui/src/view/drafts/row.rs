//! Texte d'une ligne de brouillon : état, date, destinataires et objet, tout neutralisé et raccourci.

use mira_assembly::mira_vocabulary::{AddressEntry, Draft, DraftState};

use crate::update::drafts::target::subject_label;
use crate::view::message_list::short_date::short_date;
use crate::view::safe_text::display_safe;

const RECIPIENT_CHARS: usize = 60;

/// Nom affiché de l'état d'un brouillon.
pub fn state_label(state: &DraftState) -> &'static str {
    match state {
        DraftState::Editing => "brouillon",
        DraftState::Scheduled { .. } => "planifié",
        DraftState::Sending => "en cours d'envoi",
    }
}

/// Destinataire tel qu'on le lit : « Nom <adresse> », ou l'adresse seule.
fn recipient(entry: &AddressEntry) -> String {
    match &entry.name {
        Some(name) if !name.trim().is_empty() => format!("{name} <{}>", entry.address),
        Some(_) | None => entry.address.clone(),
    }
}

/// Destinataires « À », raccourcis puis neutralisés ; « (aucun destinataire) » s'il n'y en a pas.
fn recipients(to: &[AddressEntry]) -> String {
    if to.is_empty() {
        return "(aucun destinataire)".to_string();
    }
    let joined = to.iter().map(recipient).collect::<Vec<_>>().join(", ");
    let cut: String = joined.chars().take(RECIPIENT_CHARS).collect();
    display_safe(&cut)
}

/// Ligne du brouillon : état, date de modification, destinataires, objet.
pub fn draft_text(draft: &Draft) -> String {
    format!(
        "{:<16} {}  À : {} — {}",
        state_label(&draft.state),
        short_date(draft.updated_at),
        recipients(&draft.to),
        subject_label(&draft.subject)
    )
}
