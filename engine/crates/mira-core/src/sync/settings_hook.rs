//! Accroche de la synchronisation aux réglages du moteur : comptes dont les arrivées ne sont pas notifiées.

use crate::sync::context::SyncContext;

/// Vrai si le compte est en sourdine ; une lecture impossible ne réduit pas les notifications
pub fn silenced_by_settings(ctx: &SyncContext) -> bool {
    let settings = ctx.db.read(|tx| tx.get_engine_settings());
    settings.is_ok_and(|current| current.mutes(ctx.account_id()))
}
