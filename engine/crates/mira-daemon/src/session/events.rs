//! Flux d'événements d'une connexion : file bornée, un événement par crédit, `lagged` si le client ne suit pas.

use std::sync::Arc;

use mira_protocol::{EndReason, Frame};
use serde_json::Value;
use tokio::sync::mpsc;

use super::stop::Stop;
use super::stream::Stream;
use super::{Exchanges, Outbox};
use crate::server::ServerContext;

/// Lance le flux : la tâche d'hôte remplit la file, l'expéditeur envoie une trame par crédit.
pub fn spawn_events(
    ctx: &Arc<ServerContext>,
    out: &Outbox,
    exchanges: &Exchanges,
    id: u64,
    after: Option<u64>,
    stream: Stream,
) {
    let (queue_tx, queue_rx) = mpsc::channel(ctx.event_queue.max(1));
    let pump = tokio::spawn(pump_into_queue(
        ctx.clone(),
        stream.clone(),
        queue_tx,
        after,
    ));
    let (out, exchanges) = (out.clone(), exchanges.clone());
    tokio::spawn(async move {
        let reason = forward(queue_rx, &stream, &out, id).await;
        pump.abort();
        exchanges.finish(id);
        let _ = out.send(Frame::End { id, reason }).await;
    });
}

/// Pousse chaque événement dans la file ; file pleine au-delà du délai de grâce, le client est en retard et le flux s'arrête.
async fn pump_into_queue(
    ctx: Arc<ServerContext>,
    stream: Stream,
    queue: mpsc::Sender<String>,
    after: Option<u64>,
) {
    let grace = ctx.event_lag_grace;
    ctx.host
        .pump_events_async(after, |line| {
            let (queue, stream) = (queue.clone(), stream.clone());
            async move {
                let kept = queue.send_timeout(line, grace).await.is_ok();
                if !kept {
                    stream.stop(Stop::Lagged);
                }
                kept
            }
        })
        .await;
}

/// Envoie les événements de la file au rythme des crédits ; rend la raison de la fin du flux.
async fn forward(
    mut queue: mpsc::Receiver<String>,
    stream: &Stream,
    out: &Outbox,
    id: u64,
) -> EndReason {
    loop {
        let line = tokio::select! {
            biased;
            why = stream.stopped() => return why.into(),
            line = queue.recv() => line,
        };
        let Some(line) = line else {
            return stream.reason().map_or(EndReason::Finished, Into::into);
        };
        if let Err(why) = stream.take_credit().await {
            return why.into();
        }
        let event: Value = serde_json::from_str(&line).unwrap_or(Value::Null);
        if out.send(Frame::Event { id, event }).await.is_err() {
            return EndReason::Cancelled;
        }
    }
}
