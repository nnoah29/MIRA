//! Types de contenu qu'un fournisseur d'IA peut recevoir, et parties de contexte étiquetées.

use serde::{Deserialize, Serialize};

/// Nature d'un contenu joint à une requête ; seul `MailMessage` est produit aujourd'hui.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentKind {
    MailMessage,
    MailList,
    Attachment,
    ToolResult,
}

impl ContentKind {
    /// Nom stable du type, tel qu'écrit dans les balises de contexte.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MailMessage => "mail_message",
            Self::MailList => "mail_list",
            Self::Attachment => "attachment",
            Self::ToolResult => "tool_result",
        }
    }
}

/// Partie de contexte typée, rendue par le moteur dans le dernier message de l'utilisateur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextPart {
    pub kind: ContentKind,
    pub label: String,
    pub text: String,
}
