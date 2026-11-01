//! Actions sur la boîte que l'IA propose ; l'utilisateur les confirme et l'application les exécute.

use serde::{Deserialize, Serialize};

/// Ce que l'IA peut proposer de faire à des messages ; jamais de suppression définitive ni d'envoi.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MailActionKind {
    Archive,
    Trash,
    MarkRead,
    MarkUnread,
    Star,
    Unstar,
}

/// Action proposée sur des messages, validée par le moteur : rien n'est changé avant la confirmation de l'utilisateur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActionProposal {
    pub action: MailActionKind,
    pub messages: Vec<String>,
    pub reason: Option<String>,
}
