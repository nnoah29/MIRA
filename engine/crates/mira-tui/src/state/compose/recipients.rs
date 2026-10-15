//! Contrôle des destinataires avant l'envoi : tous les champs d'adresses, modifiés ou non.

use super::addresses::parse_addresses;
use super::field::Field;
use super::state::ComposeState;

const ADDRESS_FIELDS: [Field; 3] = [Field::To, Field::Cc, Field::Bcc];

/// Verdict sur les destinataires : nombre d'adresses valides, aucun, ou première saisie invalide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecipientCheck {
    Ready(usize),
    Empty,
    Invalid(Field, String),
}

impl ComposeState {
    /// Analyse À, Cc et Cci tels qu'affichés ; la première adresse invalide l'emporte sur tout le reste.
    pub fn check_recipients(&self) -> RecipientCheck {
        let mut count = 0;
        for field in ADDRESS_FIELDS {
            match parse_addresses(&self.text(field)) {
                Ok(entries) => count += entries.len(),
                Err(segment) => return RecipientCheck::Invalid(field, segment),
            }
        }
        if count == 0 {
            return RecipientCheck::Empty;
        }
        RecipientCheck::Ready(count)
    }
}
