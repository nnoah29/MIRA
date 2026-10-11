//! Dépendances injectées d'un test : base, horloge manuelle, identifiants séquentiels, enregistreur d'événements.

use mira_core::actions::QueueContext;
use mira_core::events::RecordingSink;
use mira_core::ports::{Clock, ManualClock, SequentialIds};
use mira_core::storage::Database;
use mira_vocabulary::Timestamp;

use super::seed::seed_world;

/// Instant de départ de l'horloge des tests.
pub const START_MILLIS: i64 = 1_000_000;

/// Tout ce qu'un test de file d'actions ou de brouillons manipule.
#[derive(Debug)]
pub struct World {
    pub db: Database,
    pub clock: ManualClock,
    pub ids: SequentialIds,
    pub sink: RecordingSink,
}

impl World {
    /// Monde garni : comptes acc-1 et acc-2, boîtes, messages msg-1 à msg-3 non lus
    pub fn seeded() -> Self {
        let db = Database::open_in_memory().unwrap();
        seed_world(&db);
        Self {
            db,
            clock: ManualClock::new(Timestamp::from_millis(START_MILLIS)),
            ids: SequentialIds::new(),
            sink: RecordingSink::default(),
        }
    }

    /// Instant courant de l'horloge manuelle
    pub fn now(&self) -> Timestamp {
        self.clock.now()
    }

    /// Contexte d'exécution des opérations locales
    pub fn ctx(&self) -> QueueContext<'_> {
        QueueContext {
            db: &self.db,
            clock: &self.clock,
            ids: &self.ids,
            events: &self.sink,
        }
    }
}
