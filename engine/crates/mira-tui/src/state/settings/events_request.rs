//! Lecture de page du journal attendue : une seule à la fois, la dernière demandée.

use mira_assembly::mira_vocabulary::EventSeq;

use super::SettingsState;
use super::events::{PageMove, PendingPage};

impl SettingsState {
    /// Note la lecture d'une page après ce numéro et rend son jeton ; une lecture déjà attendue est remplacée.
    pub fn begin_events(
        &mut self,
        after: Option<EventSeq>,
        movement: PageMove,
        seeking_end: bool,
    ) -> u64 {
        let token = self.tokens.next();
        self.events.pending = Some(PendingPage {
            token,
            after,
            movement,
            seeking_end,
        });
        token
    }

    /// Rend (et oublie) la lecture attendue sous ce jeton ; une réponse périmée ne rend rien.
    pub fn take_events(&mut self, token: u64) -> Option<PendingPage> {
        match self.events.pending {
            Some(page) if page.token == token => self.events.pending.take(),
            Some(_) | None => None,
        }
    }

    /// Vrai tant qu'une lecture de page attend le moteur.
    pub fn events_loading(&self) -> bool {
        self.events.pending.is_some()
    }
}
