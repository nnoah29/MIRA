//! Liste des extensions que le faux serveur sait annoncer.

/// Extension IMAP que le faux serveur peut annoncer et honorer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Capability {
    Idle,
    Condstore,
    Qresync,
    Move,
    UidPlus,
    SpecialUse,
    Enable,
    Id,
    Gmail,
    Literalplus,
    Esearch,
}
