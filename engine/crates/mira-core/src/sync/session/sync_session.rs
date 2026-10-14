//! Une synchronisation en cours : contexte, connexion de travail et identifiant d'opération.

use mira_vocabulary::OperationId;

use super::connection::Connection;
use crate::sync::context::SyncContext;
use crate::sync::report::ReportHandle;

/// État partagé par les étapes d'une synchronisation sur une connexion de travail.
#[derive(Debug)]
pub struct SyncSession {
    pub ctx: SyncContext,
    pub conn: Connection,
    pub operation: OperationId,
    pub report: ReportHandle,
    forced_operation: Option<OperationId>,
}

impl SyncSession {
    /// Prépare une session avec un nouvel identifiant d'opération.
    pub fn new(ctx: SyncContext, conn: Connection) -> Self {
        let operation = OperationId::from_raw(ctx.new_id());
        Self {
            ctx,
            conn,
            operation,
            report: ReportHandle::default(),
            forced_operation: None,
        }
    }

    /// Relie la session au rapport de la tâche du compte.
    pub fn with_report(mut self, report: ReportHandle) -> Self {
        self.report = report;
        self
    }

    /// Impose l'identifiant d'opération de la prochaine passe (celui que le client a reçu).
    pub fn force_operation(&mut self, operation: OperationId) {
        self.forced_operation = Some(operation);
    }

    /// Prend l'identifiant d'opération imposé (sinon en fabrique un) et relit le compte (plage, réglages) avant une nouvelle passe.
    pub fn begin_pass(&mut self) {
        let fresh = OperationId::from_raw(self.ctx.new_id());
        self.operation = self.forced_operation.take().unwrap_or(fresh);
        let id = self.ctx.account.id.clone();
        if let Ok(Some(fresh)) = self.ctx.db.read(|tx| tx.get_account(&id)) {
            self.ctx.account = fresh;
        }
    }
}
