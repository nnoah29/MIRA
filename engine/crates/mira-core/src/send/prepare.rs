//! Préparation du message : le brouillon, son identité et ses pièces jointes deviennent un message MIME (10.8).

use mira_vocabulary::{AddressEntry, Draft, DraftId, MiraError, Timestamp};

use crate::actions::QueueContext;
use crate::actions::errors::not_found::draft_not_found;
use crate::drafts::draft_threading_headers;
use crate::drafts::identity::resolve_identity;
use crate::smtp::compose::{BuiltMessage, ComposeAttachment, ComposeInput, build_message};
use crate::storage::Tx;

/// Assemble le message du brouillon avec le `Message-ID` donné ; la date est l'instant de la préparation
pub(crate) fn compose_draft(
    ctx: &QueueContext<'_>,
    draft: &DraftId,
    message_id: &str,
) -> Result<BuiltMessage, MiraError> {
    let now = ctx.clock.now();
    let input = ctx
        .db
        .read(|tx| compose_input(tx, draft, message_id, now))?;
    build_message(&input)
}

/// Lit le brouillon, son identité d'envoi, ses en-têtes de fil et ses pièces jointes
fn compose_input(
    tx: &Tx<'_>,
    id: &DraftId,
    message_id: &str,
    date: Timestamp,
) -> Result<ComposeInput, MiraError> {
    let draft = tx.get_draft(id)?.ok_or_else(|| draft_not_found(id))?;
    let identity = resolve_identity(tx, &draft.account, Some(&draft.identity))?;
    let threading = draft_threading_headers(tx, &draft)?;
    Ok(ComposeInput {
        from: AddressEntry {
            name: Some(identity.display_name).filter(|name| !name.is_empty()),
            address: identity.address.as_str().to_string(),
        },
        to: draft.to.clone(),
        cc: draft.cc.clone(),
        bcc: draft.bcc.clone(),
        subject: draft.subject.clone(),
        date,
        message_id: message_id.to_string(),
        in_reply_to: threading.in_reply_to,
        references: threading.references,
        body_text: draft.body_text.clone(),
        body_html: draft.body_html.clone(),
        attachments: attachments_of(tx, &draft)?,
    })
}

/// Pièces jointes du brouillon avec leur contenu, dans l'ordre d'ajout
fn attachments_of(tx: &Tx<'_>, draft: &Draft) -> Result<Vec<ComposeAttachment>, MiraError> {
    let positions = tx.list_draft_attachment_positions(&draft.id)?;
    let mut attachments = Vec::new();
    for (position, meta) in positions.iter().zip(&draft.attachments) {
        attachments.push(ComposeAttachment {
            filename: meta.filename.clone(),
            mime_type: meta.mime_type.clone(),
            data: tx
                .read_draft_attachment_data(&draft.id, *position)?
                .unwrap_or_default(),
        });
    }
    Ok(attachments)
}
