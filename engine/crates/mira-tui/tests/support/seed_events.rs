//! Amorçage d'événements anciens dans le journal du profil, sans les diffuser (historique d'avant la session).

use mira_assembly::mira_vocabulary::{EventKind, MiraError, Timestamp};

use super::world::TestWorld;

impl TestWorld {
    /// Ajoute cet événement au journal persistant, daté d'avant la session.
    pub fn seed_event(&self, kind: &EventKind) -> Result<(), MiraError> {
        self.open_mirror()?.write(|tx| {
            tx.append_event(Timestamp::from_millis(1_000), kind)?;
            Ok(())
        })
    }
}
