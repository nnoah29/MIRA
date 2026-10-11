//! Rôles des boîtes propres au service, d'après le nom complet de la boîte.

use mira_vocabulary::MailboxRole;

const NAMES: &[(&str, MailboxRole)] = &[
    ("sent", MailboxRole::Sent),
    ("draft", MailboxRole::Drafts),
    ("trash", MailboxRole::Trash),
    ("bulk mail", MailboxRole::Junk),
    ("bulk", MailboxRole::Junk),
    ("archive", MailboxRole::Archive),
];

/// Donne le rôle d'une boîte du service d'après son nom, sans tenir compte de la casse.
pub fn role_for_name(name: &str) -> Option<MailboxRole> {
    let lowered = name.to_lowercase();
    NAMES
        .iter()
        .find(|(known, _)| *known == lowered)
        .map(|(_, role)| *role)
}
