//! Rôle déduit des noms usuels de dossiers, en plusieurs langues (dernier segment du chemin).

use mira_vocabulary::MailboxRole;

const SENT: &[&str] = &[
    "sent",
    "sent items",
    "sent messages",
    "envoyés",
    "éléments envoyés",
    "gesendet",
    "inviati",
    "enviados",
];
const DRAFTS: &[&str] = &["drafts", "brouillons", "entwürfe", "bozze", "borradores"];
const TRASH: &[&str] = &[
    "trash",
    "deleted",
    "deleted items",
    "deleted messages",
    "corbeille",
    "papierkorb",
    "cestino",
    "papelera",
];
const JUNK: &[&str] = &[
    "junk",
    "spam",
    "junk e-mail",
    "courrier indésirable",
    "indésirables",
];
const ARCHIVE: &[&str] = &["archive", "archives", "archiv"];

const TABLE: &[(&[&str], MailboxRole)] = &[
    (SENT, MailboxRole::Sent),
    (DRAFTS, MailboxRole::Drafts),
    (TRASH, MailboxRole::Trash),
    (JUNK, MailboxRole::Junk),
    (ARCHIVE, MailboxRole::Archive),
];

/// Isole le dernier segment du chemin d'après le séparateur de la hiérarchie.
fn last_segment<'a>(name: &'a str, delimiter: Option<&str>) -> &'a str {
    match delimiter {
        Some(separator) if !separator.is_empty() => name.rsplit(separator).next().unwrap_or(name),
        _ => name,
    }
}

/// Donne le rôle d'un nom usuel, sans tenir compte de la casse ; `None` si le nom est inconnu.
pub fn role_from_common_name(name: &str, delimiter: Option<&str>) -> Option<MailboxRole> {
    let segment = last_segment(name, delimiter).trim().to_lowercase();
    TABLE
        .iter()
        .find(|(names, _)| names.contains(&segment.as_str()))
        .map(|(_, role)| *role)
}
