//! Champs initiaux d'un brouillon selon son mode : neuf, réponse, réponse à tous, transfert.

use mira_vocabulary::{AddressEntry, MessageId, MessageSummary};

use crate::drafts::compose::recipients::{reply_all, reply_to};
use crate::drafts::compose::subject::{forward_subject, reply_subject};

/// Façon de composer un brouillon à partir (ou non) d'un message d'origine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Mode {
    Reply,
    ReplyAll,
    Forward,
}

/// Destinataires, objet et lien vers le message d'origine d'un brouillon qui vient d'être créé.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct DraftFields {
    pub(super) to: Vec<AddressEntry>,
    pub(super) cc: Vec<AddressEntry>,
    pub(super) subject: String,
    pub(super) in_reply_to: Option<MessageId>,
}

/// Champs d'un brouillon construit depuis le message d'origine ; `own` liste les adresses de l'utilisateur
pub(super) fn fields_from(mode: Mode, original: &MessageSummary, own: &[String]) -> DraftFields {
    let link = Some(original.id.clone());
    match mode {
        Mode::Reply => DraftFields {
            to: reply_to(original, own),
            cc: Vec::new(),
            subject: reply_subject(&original.subject),
            in_reply_to: link,
        },
        Mode::ReplyAll => {
            let recipients = reply_all(original, own);
            DraftFields {
                to: recipients.to,
                cc: recipients.cc,
                subject: reply_subject(&original.subject),
                in_reply_to: link,
            }
        }
        Mode::Forward => DraftFields {
            to: Vec::new(),
            cc: Vec::new(),
            subject: forward_subject(&original.subject),
            in_reply_to: link,
        },
    }
}
