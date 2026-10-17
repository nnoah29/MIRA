//! Point d'émission des événements, injecté dans les modules qui en produisent.

use mira_vocabulary::{Event, EventKind, MiraError};

/// Destination des événements du moteur (bus réel, ou enregistreur dans les tests).
pub trait EventSink: Send + Sync + std::fmt::Debug {
    /// Numérote, persiste et diffuse un événement
    fn emit(&self, kind: EventKind) -> Result<Event, MiraError>;
}
