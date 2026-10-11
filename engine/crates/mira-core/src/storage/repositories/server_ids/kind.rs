//! Nature d'un identifiant serveur.

/// Nature d'un identifiant serveur : stable (annoncé par le serveur) ou dérivé de l'UID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServerIdKind {
    Stable,
    Uid,
}

impl ServerIdKind {
    /// Donne le nom stocké de la nature
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Uid => "uid",
        }
    }
}
