//! Opérations d'hôte, hors du contrat d'API : arrêt du démon et nouvel essai d'ouverture du trousseau.

use std::sync::Arc;

use mira_host::envelope::success;
use mira_protocol::{Frame, HostOp};
use serde_json::Value;

use super::Outbox;
use crate::server::ServerContext;
use crate::shutdown;

/// Répond `result` ok, sans donnée, à une opération d'hôte.
async fn reply_ok(out: &Outbox, id: u64) -> bool {
    let envelope = serde_json::from_str(&success(Value::Null)).unwrap_or(Value::Null);
    out.send(Frame::Result { id, envelope }).await.is_ok()
}

/// Exécute l'opération : `shutdown` répond d'abord puis arrête dans une tâche à part, `retry_unlock` réveille l'attente du trousseau.
pub async fn run_host_op(ctx: &Arc<ServerContext>, out: &Outbox, id: u64, op: HostOp) -> bool {
    match op {
        HostOp::Shutdown => {
            let sent = reply_ok(out, id).await;
            tokio::spawn(shutdown::run(ctx.clone()));
            sent
        }
        HostOp::RetryUnlock => {
            ctx.retry_unlock.notify_one();
            reply_ok(out, id).await
        }
    }
}
