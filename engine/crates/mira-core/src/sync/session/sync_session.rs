//! Une synchronisation en cours : contexte, connexion de travail et identifiant d'opération.

use mira_vocabulary::OperationId;

use super::connection::Connection;
use crate::sync::context::SyncContext;

/// État partagé par les étapes d'une synchronisation sur une connexion de travail.
#[derive(Debug)]
pub struct SyncSession {
    pub ctx: SyncContext,
    pub conn: Connection,
    pub operation: OperationId,
}

impl SyncSession {
    /// Prépare une session avec un nouvel identifiant d'opération.
    pub fn new(ctx: SyncContext, conn: Connection) -> Self {
        let operation = OperationId::from_raw(ctx.new_id());
        Self {
            ctx,
            conn,
            operation,
        }
    }

    /// Renouvelle l'identifiant d'opération et relit le compte (plage, réglages) avant une nouvelle passe.
    pub fn begin_pass(&mut self) {
        self.operation = OperationId::from_raw(self.ctx.new_id());
        let id = self.ctx.account.id.clone();
        if let Ok(Some(fresh)) = self.ctx.db.read(|tx| tx.get_account(&id)) {
            self.ctx.account = fresh;
        }
    }
}
