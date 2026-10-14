//! Nom, côté serveur, de la boîte qui porte un rôle (Envoyés, Brouillons) d'après le miroir.

use mira_vocabulary::{MailboxRole, MiraError};

use crate::actions::replay::ReplayContext;

/// Nom de la première boîte du compte qui porte le rôle, ou rien si le miroir n'en connaît pas
pub(crate) fn role_mailbox_name(
    ctx: &ReplayContext<'_>,
    role: MailboxRole,
) -> Result<Option<String>, MiraError> {
    let found = ctx
        .queue
        .db
        .read(|tx| tx.find_mailbox_by_role(ctx.account, role))?;
    Ok(found.map(|mailbox| mailbox.name))
}
