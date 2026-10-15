//! Ouverture de l'éditeur sur le brouillon rendu par le moteur.

use mira_assembly::mira_vocabulary::{AddressEntry, Draft};

use super::addresses::format_addresses;
use super::field::{FIELD_COUNT, Field};
use super::sending::SendPhase;
use super::state::{ComposeState, FieldBuffer};
use super::text_buffer::TextBuffer;

/// Champ d'une ligne : tampon, texte de référence pris sur ce que le tampon montre, aucun signalement.
fn line_field(text: &str) -> FieldBuffer {
    let buffer = TextBuffer::single_line(text);
    FieldBuffer {
        baseline: buffer.text(),
        buffer,
        flagged: false,
    }
}

/// Champ du corps : plusieurs lignes, curseur au début.
fn body_field(text: &str) -> FieldBuffer {
    let buffer = TextBuffer::new(text);
    FieldBuffer {
        baseline: buffer.text(),
        buffer,
        flagged: false,
    }
}

/// Champ d'adresses écrit comme on le tape.
fn address_field(entries: &[AddressEntry]) -> FieldBuffer {
    line_field(&format_addresses(entries))
}

impl ComposeState {
    /// Éditeur sur ce brouillon ; le focus va aux destinataires s'il n'y en a pas, au corps sinon.
    pub fn from_draft(draft: &Draft) -> ComposeState {
        let fields: [FieldBuffer; FIELD_COUNT] = [
            address_field(&draft.to),
            address_field(&draft.cc),
            address_field(&draft.bcc),
            line_field(&draft.subject),
            body_field(&draft.body_text),
        ];
        ComposeState {
            focus: if draft.to.is_empty() {
                Field::To
            } else {
                Field::Body
            },
            send_key: None,
            draft: draft.id.clone(),
            account: draft.account.clone(),
            fields,
            saving: None,
            saved_once: false,
            close_after_save: false,
            then_quit: false,
            had_html: draft.body_html.is_some(),
            send_phase: SendPhase::Idle,
        }
    }
}
