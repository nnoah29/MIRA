//! Rôle annoncé par le serveur au moyen des attributs SPECIAL-USE (RFC 6154).

use mira_vocabulary::MailboxRole;

const ATTRIBUTES: &[(&str, MailboxRole)] = &[
    ("\\sent", MailboxRole::Sent),
    ("\\drafts", MailboxRole::Drafts),
    ("\\trash", MailboxRole::Trash),
    ("\\junk", MailboxRole::Junk),
    ("\\archive", MailboxRole::Archive),
    ("\\all", MailboxRole::All),
    ("\\flagged", MailboxRole::Flagged),
];

/// Donne le rôle du premier attribut SPECIAL-USE reconnu, sans tenir compte de la casse.
pub fn role_from_special_use(attributes: &[String]) -> Option<MailboxRole> {
    attributes.iter().find_map(|attribute| {
        let lowered = attribute.to_lowercase();
        ATTRIBUTES
            .iter()
            .find(|(known, _)| *known == lowered)
            .map(|(_, role)| *role)
    })
}
