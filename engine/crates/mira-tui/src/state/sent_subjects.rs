//! Objets des envois partis de cette session, pour annoncer leur avancement par leur nom (mémoire bornée).

use mira_assembly::mira_vocabulary::ActionId;

const REMEMBERED: usize = 64;

/// Dernières paires (action d'envoi, objet), les plus anciennes oubliées au-delà de 64.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SentSubjects {
    items: Vec<(ActionId, String)>,
}

impl SentSubjects {
    /// Retient l'objet de cet envoi.
    pub fn remember(&mut self, action: ActionId, subject: String) {
        if self.items.len() >= REMEMBERED {
            self.items.remove(0);
        }
        self.items.push((action, subject));
    }

    /// Objet de cet envoi s'il est parti de cette session.
    pub fn subject_of(&self, action: &ActionId) -> Option<&str> {
        self.items
            .iter()
            .find(|(known, _)| known == action)
            .map(|(_, subject)| subject.as_str())
    }
}
