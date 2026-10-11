//! Modification du contenu d'un brouillon en cours d'édition.

use mira_vocabulary::{
    AddressEntry, Draft, DraftId, DraftState, EmailAddress, IdentityId, MiraError,
};

use super::identity::resolve_identity;
use crate::actions::QueueContext;
use crate::actions::errors::incompatible::draft_not_editable;
use crate::actions::errors::not_found::draft_not_found;

/// Champs à changer ; un champ absent reste tel quel (`body_html: Some(None)` efface la version HTML).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DraftEdit {
    pub identity: Option<IdentityId>,
    pub to: Option<Vec<AddressEntry>>,
    pub cc: Option<Vec<AddressEntry>>,
    pub bcc: Option<Vec<AddressEntry>>,
    pub subject: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<Option<String>>,
}

/// Applique les changements à un brouillon en édition (sinon `draft_not_editable`) et le rend à jour
pub fn edit_draft(
    ctx: &QueueContext<'_>,
    id: &DraftId,
    edit: DraftEdit,
) -> Result<Draft, MiraError> {
    validate_addresses(&edit)?;
    ctx.db.write(|tx| {
        let mut draft = tx.get_draft(id)?.ok_or_else(|| draft_not_found(id))?;
        if draft.state != DraftState::Editing {
            return Err(draft_not_editable());
        }
        if let Some(wanted) = &edit.identity {
            draft.identity = resolve_identity(tx, &draft.account, Some(wanted))?.id;
        }
        apply_fields(&mut draft, edit);
        draft.updated_at = ctx.clock.now();
        tx.update_draft(&draft)?;
        Ok(draft)
    })
}

/// Copie dans le brouillon les champs présents dans la modification
fn apply_fields(draft: &mut Draft, edit: DraftEdit) {
    if let Some(to) = edit.to {
        draft.to = to;
    }
    if let Some(cc) = edit.cc {
        draft.cc = cc;
    }
    if let Some(bcc) = edit.bcc {
        draft.bcc = bcc;
    }
    if let Some(subject) = edit.subject {
        draft.subject = subject;
    }
    if let Some(body_text) = edit.body_text {
        draft.body_text = body_text;
    }
    if let Some(body_html) = edit.body_html {
        draft.body_html = body_html;
    }
}

/// Refuse une adresse de destinataire mal formée (`invalid_input.address`)
fn validate_addresses(edit: &DraftEdit) -> Result<(), MiraError> {
    let lists = [&edit.to, &edit.cc, &edit.bcc];
    for entry in lists.into_iter().flatten().flatten() {
        EmailAddress::parse(&entry.address)?;
    }
    Ok(())
}
