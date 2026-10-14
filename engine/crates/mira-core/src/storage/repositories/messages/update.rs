//! Mises à jour d'un message : drapeaux, état du contenu, fil.

use mira_vocabulary::{ContentState, Flags, MessageId, MiraError, ThreadId};
use rusqlite::params;

use super::content_state::content_state_name;
use super::errors::require_change;
use crate::storage::Tx;
use crate::storage::errors::storage_error;

impl Tx<'_> {
    /// Remplace les drapeaux du message (D-14 : ils vivent sur le message, pas sur l'apparition)
    pub fn update_message_flags(&self, id: &MessageId, flags: Flags) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE messages SET seen = ?2, flagged = ?3, answered = ?4, forwarded = ?5 WHERE id = ?1",
                params![
                    id.as_str(),
                    flags.seen,
                    flags.flagged,
                    flags.answered,
                    flags.forwarded
                ],
            )
            .map_err(storage_error)?;
        require_change(changed, id)
    }

    /// Change l'état du contenu (en-têtes seuls ou corps présent)
    pub fn set_content_state(&self, id: &MessageId, state: ContentState) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE messages SET content_state = ?2 WHERE id = ?1",
                params![id.as_str(), content_state_name(state)],
            )
            .map_err(storage_error)?;
        require_change(changed, id)
    }

    /// Enregistre l'aperçu et la présence de pièces jointes découverts à la lecture du corps
    pub fn set_message_body_summary(
        &self,
        id: &MessageId,
        preview: &str,
        has_attachments: bool,
    ) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE messages SET preview = ?2, has_attachments = ?3 WHERE id = ?1",
                params![id.as_str(), preview, has_attachments],
            )
            .map_err(storage_error)?;
        require_change(changed, id)
    }

    /// Rattache le message à un fil, ou le détache
    pub fn set_thread(&self, id: &MessageId, thread: Option<&ThreadId>) -> Result<(), MiraError> {
        let changed = self
            .sql()
            .execute(
                "UPDATE messages SET thread_id = ?2 WHERE id = ?1",
                params![id.as_str(), thread.map(ThreadId::as_str)],
            )
            .map_err(storage_error)?;
        require_change(changed, id)
    }
}
