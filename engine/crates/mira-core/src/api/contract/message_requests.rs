//! Requêtes du domaine « lecture des messages » (12.4) : listes, corps, pièces jointes, HTML sûr.

use mira_vocabulary::{AttachmentId, MessageId};
use serde::{Deserialize, Serialize};

use crate::reading::ListQuery;

/// Ce à quoi s'applique l'autorisation de contenu distant : un message ou un expéditeur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RemoteContentTarget {
    Message { message: MessageId },
    Sender { address: String },
}

/// Pièce jointe à enregistrer et fichier de destination.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttachmentTarget {
    pub attachment: AttachmentId,
    pub destination: String,
}

/// Lectures locales de messages et opérations réseau explicites sur leur contenu.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum MessageRequest {
    ListMessages(ListQuery),
    GetMessage { message: MessageId },
    GetMessageBody { message: MessageId },
    FetchMessageBody { message: MessageId },
    GetRawSource { message: MessageId },
    ListAttachments { message: MessageId },
    SaveAttachment(AttachmentTarget),
    GetSafeHtml { message: MessageId },
    AllowRemoteContent { scope: RemoteContentTarget },
}
