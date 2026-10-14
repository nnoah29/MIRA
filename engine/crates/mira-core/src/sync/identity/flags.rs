//! Drapeaux IMAP traduits en drapeaux du message (D-14) et en état « supprimé » de l'apparition.

use mira_vocabulary::Flags;

/// Indique si la liste contient le drapeau, sans tenir compte de la casse.
fn has(flags: &[String], name: &str) -> bool {
    flags.iter().any(|flag| flag.eq_ignore_ascii_case(name))
}

/// Drapeaux portés par le message lui-même.
pub fn server_flags(flags: &[String]) -> Flags {
    Flags {
        seen: has(flags, "\\Seen"),
        flagged: has(flags, "\\Flagged"),
        answered: has(flags, "\\Answered"),
        forwarded: has(flags, "$Forwarded"),
    }
}

/// Indique si le serveur marque le message « supprimé » dans cette boîte.
pub fn is_deleted(flags: &[String]) -> bool {
    has(flags, "\\Deleted")
}
