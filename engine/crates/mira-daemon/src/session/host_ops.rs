//! Opérations d'hôte, hors du contrat d'API : arrêt du démon, nouvel essai du trousseau, focus du client.

use std::sync::Arc;

use mira_host::envelope::success;
use mira_protocol::{Frame, HostOp};
use serde_json::Value;

use super::{Outbox, PresenceKey};
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
        HostOp::TestNotification => {
            ctx.test_notification.run();
            reply_ok(out, id).await
        }
        HostOp::RetryUnlock => {
            ctx.retry_unlock.notify_one();
            reply_ok(out, id).await
        }
    }
}

/// Exécute une trame d'hôte (`host_op`, `set_presence` ou `ping`) ; faux si ce n'en est pas une.
pub async fn run_frame(
    frame: Frame,
    out: &Outbox,
    ctx: &Arc<ServerContext>,
    presence: &PresenceKey,
) -> bool {
    match frame {
        Frame::HostOp { id, op } => run_host_op(ctx, out, id, op).await,
        Frame::SetPresence { id, focused } => {
            presence.set(focused);
            reply_ok(out, id).await
        }
        Frame::Ping { nonce } => out.send(Frame::Pong { nonce }).await.is_ok(),
        _ => false,
    }
}
