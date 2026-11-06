//! Appels du contrat : exécutés par l'hôte dans une tâche détachée, que la connexion survive ou non.

use std::sync::Arc;

use mira_protocol::Frame;
use serde_json::Value;

use super::{Exchanges, Outbox};
use crate::server::ServerContext;

/// Lance l'appel ; la tâche n'est jamais interrompue par la fermeture de la connexion (envois, synchros).
pub fn spawn_call(
    ctx: &Arc<ServerContext>,
    out: &Outbox,
    exchanges: &Exchanges,
    id: u64,
    request: Value,
) {
    let (ctx, out, exchanges) = (ctx.clone(), out.clone(), exchanges.clone());
    tokio::spawn(async move {
        let text = ctx.host.execute(&request.to_string()).await;
        exchanges.finish(id);
        let envelope = serde_json::from_str(&text).unwrap_or(Value::Null);
        let _ = out.send(Frame::Result { id, envelope }).await;
    });
}
