//! Une session par connexion : poignée de main, puis lecture des trames et écriture sérialisée des réponses.

mod access;
mod calls;
mod chat;
mod events;
mod exchanges;
mod hello;
mod host_ops;
mod open;
mod presence;
mod reply;
mod stop;
mod stream;
mod stream_control;
mod stream_wait;

use std::sync::Arc;
use std::time::Duration;

use mira_protocol::Frame;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub use access::Access;
pub use exchanges::Exchanges;
pub use presence::PresenceKey;
pub use stop::Stop;
pub use stream::Stream;

use crate::server::{ServerContext, wait_closing};
use crate::transport::{Accepted, ByteStream};
use crate::wire::{read_frame, write_frame};

/// File des trames à écrire sur la connexion, unique écrivain.
pub type Outbox = mpsc::Sender<Frame>;

type Reader = ReadHalf<Box<dyn ByteStream>>;
type Writer = WriteHalf<Box<dyn ByteStream>>;

const OUTBOX_DEPTH: usize = 64;
const FLUSH_GRACE: Duration = Duration::from_secs(1);

/// Pourquoi la lecture des trames s'est arrêtée.
enum Outcome {
    Gone,
    Violation,
    Closing,
}

/// Sert une connexion jusqu'à sa fin ; toute violation du protocole la ferme sans explication.
pub async fn run(accepted: Accepted, ctx: Arc<ServerContext>) {
    let Ok(permit) = ctx.pending_hello.clone().try_acquire_owned() else {
        return;
    };
    let (mut reader, mut writer) = tokio::io::split(accepted.stream);
    let welcome = hello::handshake(&mut reader, &mut writer, &ctx).await;
    drop(permit);
    let Some(access) = welcome else {
        return;
    };
    let _attached = ctx.clients.attach();
    let presence = PresenceKey::new(ctx.host.clone());
    let (out, queue) = mpsc::channel(OUTBOX_DEPTH);
    let mut writer_task = tokio::spawn(write_loop(writer, queue, ctx.max_frame));
    let exchanges = Exchanges::default();
    let outcome = read_loop(
        &mut reader,
        &out,
        &ctx,
        &exchanges,
        &mut writer_task,
        (access, &presence),
    )
    .await;
    exchanges.cancel_all();
    finish(outcome, out, writer_task).await;
}

/// Écrit les trames de la file l'une après l'autre jusqu'à fermeture de la file ou échec d'écriture.
async fn write_loop(mut writer: Writer, mut queue: mpsc::Receiver<Frame>, max: u32) {
    while let Some(frame) = queue.recv().await {
        if write_frame(&mut writer, &frame, max).await.is_err() {
            break;
        }
    }
}

/// Lit et aiguille les trames jusqu'à la fin de la connexion, une violation, l'arrêt du serveur ou l'échec de l'écriture.
async fn read_loop(
    reader: &mut Reader,
    out: &Outbox,
    ctx: &Arc<ServerContext>,
    exchanges: &Exchanges,
    writer_task: &mut JoinHandle<()>,
    (access, presence): (Access, &PresenceKey),
) -> Outcome {
    let mut closing = ctx.closing_receiver();
    loop {
        let read = tokio::select! {
            read = read_frame(reader, ctx.max_frame) => read,
            _ = wait_closing(&mut closing) => return Outcome::Closing,
            _ = &mut *writer_task => return Outcome::Violation,
        };
        match read {
            Ok(Some(frame)) => {
                if !access.permits(&frame) || !dispatch(frame, out, ctx, exchanges, presence).await
                {
                    return Outcome::Violation;
                }
            }
            Ok(None) => return Outcome::Gone,
            Err(_) => return Outcome::Violation,
        }
    }
}

/// Traite une trame du client ; faux si elle viole le protocole.
async fn dispatch(
    frame: Frame,
    out: &Outbox,
    ctx: &Arc<ServerContext>,
    exchanges: &Exchanges,
    presence: &PresenceKey,
) -> bool {
    match frame {
        Frame::Call { id, request } => {
            let fresh = exchanges.begin_call(id);
            if fresh {
                calls::spawn_call(ctx, out, exchanges, id, request);
            }
            fresh
        }
        Frame::HostOp { .. } | Frame::SetPresence { .. } | Frame::Ping { .. } => {
            host_ops::run_frame(frame, out, ctx, presence).await
        }
        Frame::Credit { id, n } => {
            exchanges.add_credit(id, n);
            true
        }
        Frame::Cancel { id } => {
            exchanges.cancel(id);
            true
        }
        Frame::Unknown | Frame::Pong { .. } => true,
        other => open::open_frame(other, out, ctx, exchanges),
    }
}

/// Ferme la connexion : à l'arrêt du serveur ou à la demi-fermeture du client, on laisse une seconde pour écrire ce qui est en file.
async fn finish(outcome: Outcome, out: Outbox, mut writer_task: JoinHandle<()>) {
    if matches!(outcome, Outcome::Closing | Outcome::Gone) {
        drop(out);
        let _ = tokio::time::timeout(FLUSH_GRACE, &mut writer_task).await;
    }
    writer_task.abort();
}
