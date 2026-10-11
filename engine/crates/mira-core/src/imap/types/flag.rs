//! Drapeaux de message que le moteur pose ou retire.

/// Drapeau IMAP géré par le moteur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImapFlag {
    Seen,
    Flagged,
    Answered,
    Deleted,
    Draft,
    Forwarded,
}

impl ImapFlag {
    /// Nom du drapeau tel qu'il est écrit dans une commande IMAP.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Seen => "\\Seen",
            Self::Flagged => "\\Flagged",
            Self::Answered => "\\Answered",
            Self::Deleted => "\\Deleted",
            Self::Draft => "\\Draft",
            Self::Forwarded => "$Forwarded",
        }
    }
}
