//! Resynchronisation complète d'une boîte (10.5, S13) : relit toute la plage, comme après une renumérotation.

use mira_vocabulary::{MailboxId, MiraError};

use super::start::launch;
use super::targets::{Target, mailbox_owner};
use crate::api::contract::Response;
use crate::api::engine::Engine;
use crate::sync::RequestScope;

/// Demande la resynchronisation à la tâche du compte et rend l'opération ; les actions en attente survivent
pub(crate) fn handle_resync_mailbox(
    engine: &Engine,
    mailbox: MailboxId,
) -> Result<Response, MiraError> {
    let account = mailbox_owner(engine, &mailbox)?;
    let scope = RequestScope::Resync(mailbox);
    launch(engine, "resync_mailbox", vec![Target { account, scope }])
}
