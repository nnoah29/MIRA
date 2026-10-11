//! Autorisation de charger le contenu distant d'un message ou de tous ceux d'un expéditeur.

/// Portée d'une autorisation : un message précis, ou un expéditeur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RemoteContentScope {
    Message,
    Sender,
}

impl RemoteContentScope {
    /// Donne le nom stocké de la portée
    pub(super) fn as_column(self) -> &'static str {
        match self {
            Self::Message => "message",
            Self::Sender => "sender",
        }
    }

    /// Relit une portée depuis son nom stocké
    pub(super) fn from_column(name: &str) -> Option<Self> {
        match name {
            "message" => Some(Self::Message),
            "sender" => Some(Self::Sender),
            _ => None,
        }
    }

    /// Normalise la valeur : les adresses d'expéditeur sont comparées en minuscules
    pub(super) fn normalize(self, value: &str) -> String {
        match self {
            Self::Message => value.to_string(),
            Self::Sender => value.trim().to_lowercase(),
        }
    }
}

/// Autorisation enregistrée : portée et valeur (identifiant de message ou adresse).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RemoteContentAllowance {
    pub scope: RemoteContentScope,
    pub value: String,
}
