//! Capacités du serveur que le moteur sait exploiter, indépendantes du texte brut d'IMAP.

/// Extension IMAP connue du moteur.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ServerCapability {
    Idle,
    Notify,
    Condstore,
    Qresync,
    Move,
    UidPlus,
    SpecialUse,
    ObjectId,
    Replace,
    Esearch,
    MultiSearch,
    Compress,
    LiteralPlus,
    Utf8Accept,
    Id,
    Enable,
}

const NAMES: &[(&str, ServerCapability)] = &[
    ("IDLE", ServerCapability::Idle),
    ("NOTIFY", ServerCapability::Notify),
    ("CONDSTORE", ServerCapability::Condstore),
    ("QRESYNC", ServerCapability::Qresync),
    ("MOVE", ServerCapability::Move),
    ("UIDPLUS", ServerCapability::UidPlus),
    ("SPECIAL-USE", ServerCapability::SpecialUse),
    ("OBJECTID", ServerCapability::ObjectId),
    ("REPLACE", ServerCapability::Replace),
    ("ESEARCH", ServerCapability::Esearch),
    ("MULTISEARCH", ServerCapability::MultiSearch),
    ("COMPRESS", ServerCapability::Compress),
    ("COMPRESS=DEFLATE", ServerCapability::Compress),
    ("LITERAL+", ServerCapability::LiteralPlus),
    ("UTF8=ACCEPT", ServerCapability::Utf8Accept),
    ("ID", ServerCapability::Id),
    ("ENABLE", ServerCapability::Enable),
];

impl ServerCapability {
    /// Reconnaît un nom IMAP sans tenir compte de la casse ; `None` pour une extension inconnue.
    pub fn from_imap_name(name: &str) -> Option<Self> {
        let upper = name.trim().to_ascii_uppercase();
        NAMES
            .iter()
            .find(|(known, _)| *known == upper)
            .map(|(_, capability)| *capability)
    }
}
