//! Apparition d'un message dans une boîte (D-14 : seul « supprimé » y vit).

use serde::{Deserialize, Serialize};

use crate::ids::kinds::{MailboxId, MessageId};

/// Présence d'un message dans une boîte, identifiée par son UID serveur.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Appearance {
    pub message: MessageId,
    pub mailbox: MailboxId,
    pub uid: u32,
    pub deleted: bool,
    pub needs_revalidation: bool,
}
