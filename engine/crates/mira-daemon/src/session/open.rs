//! Ouverture des échanges à identifiant : flux d'événements, flux de chat, HTML d'origine.

use std::sync::Arc;

use mira_protocol::Frame;
use serde_json::Value;

use super::reply::result_frame;
use super::stream::Stream;
use super::{Exchanges, Outbox, chat, events};
use crate::server::ServerContext;

/// Ouvre un flux d'événements ; faux si l'identifiant est déjà en cours.
pub fn open_events(
    ctx: &Arc<ServerContext>,
    out: &Outbox,
    exchanges: &Exchanges,
    id: u64,
    after: Option<u64>,
    credit: u32,
) -> bool {
    let stream = Stream::new(credit);
    let fresh = exchanges.begin_stream(id, stream.clone());
    if fresh {
        events::spawn_events(ctx, out, exchanges, id, after, stream);
    }
    fresh
}

/// Ouvre un flux de chat ; faux si l'identifiant est déjà en cours.
pub fn open_chat(
    ctx: &Arc<ServerContext>,
    out: &Outbox,
    exchanges: &Exchanges,
    id: u64,
    request: Value,
    credit: u32,
) -> bool {
    let stream = Stream::new(credit);
    let fresh = exchanges.begin_stream(id, stream.clone());
    if fresh {
        chat::spawn_chat(ctx, out, exchanges, id, request, stream);
    }
    fresh
}

/// Lit le HTML d'origine d'un message et répond par un `result` ; faux si l'identifiant est déjà en cours.
pub fn open_html(
    ctx: &Arc<ServerContext>,
    out: &Outbox,
    exchanges: &Exchanges,
    id: u64,
    message: String,
) -> bool {
    let fresh = exchanges.begin_call(id);
    if fresh {
        let (ctx, out, exchanges) = (ctx.clone(), out.clone(), exchanges.clone());
        tokio::spawn(async move {
            let text = ctx.host.original_html(&message).await;
            exchanges.finish(id);
            let _ = out.send(result_frame(id, &text, ctx.max_frame)).await;
        });
    }
    fresh
}

/// Aiguille les trames qui ouvrent un échange à identifiant ; toute autre trame viole le protocole.
pub fn open_frame(
    frame: Frame,
    out: &Outbox,
    ctx: &Arc<ServerContext>,
    exchanges: &Exchanges,
) -> bool {
    match frame {
        Frame::OpenEvents { id, after, credit } => {
            open_events(ctx, out, exchanges, id, after, credit)
        }
        Frame::OpenChat {
            id,
            request,
            credit,
        } => open_chat(ctx, out, exchanges, id, request, credit),
        Frame::GetOriginalHtml { id, message } => open_html(ctx, out, exchanges, id, message),
        _ => false,
    }
}
