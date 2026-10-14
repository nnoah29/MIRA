//! Ménage de fin de passe : les messages sans aucune apparition quittent le miroir (le serveur gagne).

use mira_vocabulary::{EventKind, MiraError};

use crate::sync::context::SyncContext;

/// Supprime les messages orphelins du compte et annonce chaque suppression.
pub fn delete_orphans(ctx: &SyncContext) -> Result<(), MiraError> {
    let gone = ctx
        .db
        .write(|tx| tx.delete_orphan_messages(&ctx.account.id))?;
    for message in gone {
        ctx.emit(EventKind::MessageDeleted {
            account: ctx.account.id.clone(),
            message,
        });
    }
    Ok(())
}
