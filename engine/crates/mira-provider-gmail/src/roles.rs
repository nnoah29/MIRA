//! Rôles des boîtes propres au service, sous les préfixes `[Gmail]` et `[Google Mail]`.

use mira_vocabulary::MailboxRole;

const PREFIXES: &[&str] = &["[gmail]", "[google mail]"];

const NAMES: &[(&str, MailboxRole)] = &[
    ("sent mail", MailboxRole::Sent),
    ("drafts", MailboxRole::Drafts),
    ("trash", MailboxRole::Trash),
    ("spam", MailboxRole::Junk),
    ("all mail", MailboxRole::All),
    ("starred", MailboxRole::Flagged),
];

/// Donne le rôle d'une boîte `[Gmail]/…` ou `[Google Mail]/…`, sans tenir compte de la casse.
pub fn role_for_name(name: &str) -> Option<MailboxRole> {
    let lowered = name.to_lowercase();
    let (prefix, rest) = lowered.split_once('/')?;
    if !PREFIXES.contains(&prefix) {
        return None;
    }
    NAMES
        .iter()
        .find(|(known, _)| *known == rest)
        .map(|(_, role)| *role)
}
