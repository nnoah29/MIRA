//! Particularités d'un service que le moteur doit connaître (D-05, D-07, D-08).

use serde::{Deserialize, Serialize};

/// Façon dont le service organise les messages.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MailboxModel {
    #[default]
    Folders,
    Labels,
}

/// Façon d'archiver un message sur ce service.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveStrategy {
    #[default]
    MoveToArchiveMailbox,
    RemoveFromInbox,
}

/// Écarts d'un service par rapport à la norme IMAP ; la valeur par défaut est la norme.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderQuirks {
    pub mailbox_model: MailboxModel,
    pub archive: ArchiveStrategy,
    pub server_saves_sent_copy: bool,
    pub unreliable_extensions: Vec<String>,
    pub extra_fetch_attributes: Vec<String>,
}
