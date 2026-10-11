//! Événement numéroté et horodaté.

use serde::{Deserialize, Serialize};

use super::kind::EventKind;
use crate::ids::keys::EventSeq;
use crate::time::Timestamp;

/// Événement du moteur : numéro croissant, instant d'émission et nature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub seq: EventSeq,
    pub at: Timestamp,
    pub kind: EventKind,
}
