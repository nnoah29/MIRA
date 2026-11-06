//! Opérations d'hôte, hors du contrat d'API : pour l'instant l'arrêt du démon.

use std::sync::Arc;

use mira_host::envelope::success;
use mira_protocol::{Frame, HostOp};
use serde_json::Value;

use super::Outbox;
use crate::server::ServerContext;
use crate::shutdown;

/// Exécute l'opération : répond d'abord `result`, puis lance l'arrêt dans une tâche à part.
pub async fn run_host_op(ctx: &Arc<ServerContext>, out: &Outbox, id: u64, op: HostOp) -> bool {
    match op {
        HostOp::Shutdown => {
            let envelope = serde_json::from_str(&success(Value::Null)).unwrap_or(Value::Null);
            let sent = out.send(Frame::Result { id, envelope }).await.is_ok();
            tokio::spawn(shutdown::run(ctx.clone()));
            sent
        }
    }
}
