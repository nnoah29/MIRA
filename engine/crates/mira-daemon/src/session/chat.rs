//! Flux de chat d'une connexion : une ligne par crédit ; sans crédit la génération n'est plus lue.

use std::sync::Arc;

use mira_protocol::{EndReason, Frame};
use serde_json::Value;

use super::stream::Stream;
use super::{Exchanges, Outbox};
use crate::server::ServerContext;

/// Lance la génération ; chaque ligne attend un crédit, et l'annulation fait rendre faux à l'émetteur.
pub fn spawn_chat(
    ctx: &Arc<ServerContext>,
    out: &Outbox,
    exchanges: &Exchanges,
    id: u64,
    request: Value,
    stream: Stream,
) {
    let (ctx, out, exchanges) = (ctx.clone(), out.clone(), exchanges.clone());
    tokio::spawn(async move {
        let emit = |line: String| {
            let (stream, out) = (stream.clone(), out.clone());
            async move { send_line(&stream, &out, id, line).await }
        };
        ctx.host.pump_chat_async(&request.to_string(), emit).await;
        let reason = stream.reason().map_or(EndReason::Finished, Into::into);
        exchanges.finish(id);
        let _ = out.send(Frame::End { id, reason }).await;
    });
}

/// Attend un crédit puis envoie la ligne ; faux si le flux est arrêté ou la connexion perdue.
async fn send_line(stream: &Stream, out: &Outbox, id: u64, line: String) -> bool {
    if stream.take_credit().await.is_err() {
        return false;
    }
    let line = serde_json::from_str(&line).unwrap_or(Value::Null);
    out.send(Frame::ChatDelta { id, line }).await.is_ok()
}
