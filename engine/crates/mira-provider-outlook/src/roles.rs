//! Rôles des boîtes propres au service, d'après le nom complet de la boîte.

use mira_vocabulary::MailboxRole;

const NAMES: &[(&str, MailboxRole)] = &[
    ("sent items", MailboxRole::Sent),
    ("deleted items", MailboxRole::Trash),
    ("junk email", MailboxRole::Junk),
    ("junk", MailboxRole::Junk),
    ("archive", MailboxRole::Archive),
    ("drafts", MailboxRole::Drafts),
];

/// Donne le rôle d'une boîte du service d'après son nom, sans tenir compte de la casse.
pub fn role_for_name(name: &str) -> Option<MailboxRole> {
    let lowered = name.to_lowercase();
    NAMES
        .iter()
        .find(|(known, _)| *known == lowered)
        .map(|(_, role)| *role)
}
