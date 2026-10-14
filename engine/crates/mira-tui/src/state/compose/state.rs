//! Brouillon ouvert dans l'éditeur : un tampon par champ, ce qui est déjà enregistré et l'enregistrement en cours.

use mira_assembly::mira_vocabulary::{AccountId, DraftId, IdempotencyKey};

use super::field::{FIELD_COUNT, Field};
use super::text_buffer::TextBuffer;

/// Tampon d'un champ, son texte au dernier enregistrement et le signalement d'une saisie invalide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldBuffer {
    pub(super) buffer: TextBuffer,
    pub(super) baseline: String,
    pub(super) flagged: bool,
}

/// Brouillon en cours d'édition ; les champs privés ne changent que par les méthodes du module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComposeState {
    pub focus: Field,
    pub send_key: Option<IdempotencyKey>,
    pub(super) draft: DraftId,
    pub(super) account: AccountId,
    pub(super) fields: [FieldBuffer; FIELD_COUNT],
    pub(super) saving: Option<Vec<(Field, String)>>,
    pub(super) saved_once: bool,
    pub(super) close_after_save: bool,
    pub(super) then_quit: bool,
    pub(super) had_html: bool,
}
