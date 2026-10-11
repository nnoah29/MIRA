//! Mots-clés CAPABILITY correspondant à chaque extension.

use super::Capability;

impl Capability {
    /// Mot-clé annoncé dans la réponse CAPABILITY
    pub(crate) fn token(self) -> &'static str {
        match self {
            Self::Idle => "IDLE",
            Self::Condstore => "CONDSTORE",
            Self::Qresync => "QRESYNC",
            Self::Move => "MOVE",
            Self::UidPlus => "UIDPLUS",
            Self::SpecialUse => "SPECIAL-USE",
            Self::Enable => "ENABLE",
            Self::Id => "ID",
            Self::Gmail => "X-GM-EXT-1",
            Self::Literalplus => "LITERAL+",
            Self::Esearch => "ESEARCH",
        }
    }
}
