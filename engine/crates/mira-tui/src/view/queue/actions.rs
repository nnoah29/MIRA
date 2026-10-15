//! Nom d'une action de la file autre qu'un envoi, et nombre de messages visés.

use mira_assembly::mira_vocabulary::ActionKind;

use super::outbox::shown;

/// L'un ou l'autre libellé selon le drapeau.
fn either(flag: bool, yes: &str, no: &str) -> String {
    if flag { yes } else { no }.to_string()
}

/// Nom de l'action ; les noms de boîtes choisis par l'utilisateur sont neutralisés.
#[rustfmt::skip]
fn kind_name(kind: &ActionKind) -> String {
    match kind {
        ActionKind::MarkRead { read } => either(*read, "Marquer lu", "Marquer non lu"),
        ActionKind::MarkFlagged { flagged } => either(*flagged, "Étoiler", "Retirer l'étoile"),
        ActionKind::MarkSpam { spam } => either(*spam, "Signaler indésirable", "Retirer des indésirables"),
        ActionKind::Move { .. } => "Déplacer".to_string(),
        ActionKind::AddToMailbox { .. } => "Ajouter à une boîte".to_string(),
        ActionKind::RemoveFromMailbox { .. } => "Retirer d'une boîte".to_string(),
        ActionKind::Archive => "Archiver".to_string(),
        ActionKind::Trash => "Mettre à la corbeille".to_string(),
        ActionKind::DeletePermanently => "Supprimer définitivement".to_string(),
        ActionKind::CreateMailbox { name, .. } => format!("Créer la boîte « {} »", shown(name)),
        ActionKind::RenameMailbox { name, .. } => format!("Renommer une boîte en « {} »", shown(name)),
        ActionKind::DeleteMailbox { .. } => "Supprimer une boîte".to_string(),
        ActionKind::EmptyMailbox { .. } => "Vider une boîte".to_string(),
        ActionKind::SaveDraft { .. } => "Enregistrer un brouillon".to_string(),
        ActionKind::Send { .. } => "Envoyer un message".to_string(),
    }
}

/// Nom de l'action suivi du nombre de messages visés, s'il y en a.
pub fn action_label(kind: &ActionKind, targets: usize) -> String {
    let name = kind_name(kind);
    match targets {
        0 => name,
        1 => format!("{name} (1 message)"),
        count => format!("{name} ({count} messages)"),
    }
}
