//! Message ouvert dans le lecteur : contenu, pièces jointes, fil et défilement.

use mira_assembly::mira_vocabulary::{Attachment, MessageId, MessageSummary};
use mira_assembly::reading::BodyView;

/// Ce que la zone de corps du lecteur montre.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ReaderContent {
    #[default]
    Empty,
    Loading,
    Body(BodyView),
    OfflineMissing,
    Failed(String),
}

/// Message ouvert et tout ce qui le complète ; vide tant qu'aucun message n'est ouvert.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReaderState {
    pub scroll: u16,
    pub content: ReaderContent,
    pub attachments: Vec<Attachment>,
    pub thread: Vec<MessageSummary>,
    message: Option<MessageSummary>,
}

impl ReaderState {
    /// Ouvre ce message : tout ce qui concernait le précédent est oublié, le corps est en attente.
    pub fn open(&mut self, message: MessageSummary) {
        *self = ReaderState {
            content: ReaderContent::Loading,
            message: Some(message),
            ..ReaderState::default()
        };
    }

    /// Message ouvert.
    pub fn message(&self) -> Option<&MessageSummary> {
        self.message.as_ref()
    }

    /// Vrai si ce message est celui qui est ouvert.
    pub fn shows(&self, id: &MessageId) -> bool {
        self.message.as_ref().is_some_and(|m| &m.id == id)
    }

    /// Dernière ligne du corps, qui borne le défilement.
    pub fn max_scroll(&self) -> u16 {
        match &self.content {
            ReaderContent::Body(body) => {
                let last = body.text.lines().count().saturating_sub(1);
                u16::try_from(last).unwrap_or(u16::MAX)
            }
            ReaderContent::Empty
            | ReaderContent::Loading
            | ReaderContent::OfflineMissing
            | ReaderContent::Failed(_) => 0,
        }
    }
}
