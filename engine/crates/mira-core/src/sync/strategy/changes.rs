//! Changements d'une boîte depuis le dernier repère, et leur application au miroir.

use mira_vocabulary::{Mailbox, MiraError, SyncMarker};

use super::announce::updated;
use super::detect::{compared_changes, qresync_changes};
use super::flag_changes::{apply_flag_changes, remove_vanished};
use super::headers::{Announce, ingest_headers};
use crate::capabilities::ChangeDetection;
use crate::imap::{FetchedFlags, SelectedMailbox};
use crate::sync::session::SyncSession;

/// Ce que le serveur a changé dans une boîte depuis le repère.
#[derive(Debug, Clone, Default)]
pub struct Changes {
    pub vanished: Vec<u32>,
    pub changed: Vec<FetchedFlags>,
    pub new: Vec<u32>,
}

/// Cherche les changements avec la stratégie retenue.
async fn detect_changes(
    session: &mut SyncSession,
    mailbox: &Mailbox,
    selected: &SelectedMailbox,
    detection: ChangeDetection,
) -> Result<Changes, MiraError> {
    let marker: SyncMarker = mailbox.marker.ok_or_else(missing_marker)?;
    match detection {
        ChangeDetection::Qresync if selected.vanished_incomplete => {
            compared_changes(session, mailbox, marker, true).await
        }
        ChangeDetection::Qresync => qresync_changes(session, selected, marker).await,
        ChangeDetection::Condstore => compared_changes(session, mailbox, marker, true).await,
        ChangeDetection::FullCompare => compared_changes(session, mailbox, marker, false).await,
    }
}

/// Erreur interne : un plan incrémental suppose un repère.
fn missing_marker() -> MiraError {
    use mira_vocabulary::ErrorCategory;
    use mira_vocabulary::error::codes::internal;
    MiraError::new(
        ErrorCategory::Internal,
        internal::BUG,
        "repère de boîte absent",
    )
}

/// Détecte puis applique les changements : disparitions, drapeaux (le serveur gagne), nouveaux messages.
pub async fn apply_changes(
    session: &mut SyncSession,
    mailbox: &Mailbox,
    selected: &SelectedMailbox,
    detection: ChangeDetection,
) -> Result<(), MiraError> {
    let changes = detect_changes(session, mailbox, selected, detection).await?;
    let ctx = session.ctx.clone();
    let updated_ids = ctx.db.write(|tx| {
        let touched = remove_vanished(tx, &mailbox.id, &changes.vanished)?;
        let mut modified = apply_flag_changes(tx, &mailbox.id, &changes.changed)?;
        modified.extend(touched.into_iter().filter(|id| exists(tx, id)));
        Ok(modified)
    })?;
    for message in &updated_ids {
        ctx.emit(updated(&ctx.account.id, message));
    }
    let validity = selected.uid_validity;
    ingest_headers(session, mailbox, validity, &changes.new, Announce::Live).await
}

/// Le message a-t-il encore au moins une apparition ?
fn exists(tx: &crate::storage::Tx<'_>, id: &mira_vocabulary::MessageId) -> bool {
    tx.appearances_of_message(id)
        .is_ok_and(|found| !found.is_empty())
}
