//! Pièce jointe d'un message reçu et son état de cache.

use serde::{Deserialize, Serialize};

use crate::ids::kinds::{AttachmentId, MessageId};

/// Présence locale du contenu d'une pièce jointe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheState {
    NotDownloaded,
    Cached,
}

/// Pièce jointe d'un message (métadonnées seulement).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Attachment {
    pub id: AttachmentId,
    pub message: MessageId,
    pub filename: Option<String>,
    pub mime_type: String,
    pub size: u64,
    pub content_id: Option<String>,
    pub inline: bool,
    pub cache: CacheState,
}
