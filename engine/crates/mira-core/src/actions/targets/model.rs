//! Cibles serveur mémorisées à la soumission : où se trouvaient les messages et les boîtes visés (colonne `server_targets_json`).

use mira_vocabulary::{MailboxId, MessageId};
use serde::{Deserialize, Serialize};

/// Une apparition du message sur le serveur ; `uid` vaut 0 si l'apparition n'est encore que locale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppearanceTarget {
    pub mailbox: MailboxId,
    pub mailbox_name: String,
    pub uid_validity: Option<u32>,
    pub uid: u32,
}

/// Un message visé : identifiant stable du serveur (s'il en a un) et toutes ses apparitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageTarget {
    pub message: MessageId,
    pub stable_id: Option<String>,
    pub appearances: Vec<AppearanceTarget>,
}

/// Une boîte visée, avec le nom qu'elle portait sur le serveur au moment de la soumission.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MailboxTarget {
    pub mailbox: MailboxId,
    pub name: String,
    pub uid_validity: Option<u32>,
}

/// Tout ce que le rejeu doit savoir pour viser le serveur sans relire un miroir qui a pu changer.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerTargets {
    pub messages: Vec<MessageTarget>,
    pub mailboxes: Vec<MailboxTarget>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sent_message_id: Option<String>,
}
