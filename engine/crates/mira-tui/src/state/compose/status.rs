//! État d'enregistrement du brouillon : champs modifiés, signalements, enregistrement en cours.

use super::field::Field;
use super::state::ComposeState;

impl ComposeState {
    /// Vrai si le texte de ce champ diffère de celui du dernier enregistrement.
    pub fn is_dirty(&self, field: Field) -> bool {
        let slot = &self.fields[field.index()];
        !slot.buffer.matches(&slot.baseline)
    }

    /// Vrai si un champ diffère à la fois de son dernier enregistrement et du texte d'un enregistrement en vol.
    pub fn has_dirty_fields(&self) -> bool {
        let in_flight = self.saving.as_deref().unwrap_or_default();
        Field::ALL.into_iter().any(|field| {
            let slot = &self.fields[field.index()];
            let sent = in_flight
                .iter()
                .any(|(f, text)| *f == field && slot.buffer.matches(text));
            self.is_dirty(field) && !sent
        })
    }

    /// Vrai si des modifications ne sont pas (ou pas encore) enregistrées.
    pub fn has_unsaved(&self) -> bool {
        self.saving.is_some() || Field::ALL.into_iter().any(|field| self.is_dirty(field))
    }

    /// Vrai si la saisie de ce champ a été refusée au dernier enregistrement et n'a pas été corrigée.
    pub fn is_flagged(&self, field: Field) -> bool {
        self.fields[field.index()].flagged
    }

    /// Vrai tant que la réponse du moteur à un enregistrement n'est pas arrivée.
    pub fn is_saving(&self) -> bool {
        self.saving.is_some()
    }

    /// Vrai si l'utilisateur a déjà enregistré ce brouillon.
    pub fn saved_once(&self) -> bool {
        self.saved_once
    }
}
