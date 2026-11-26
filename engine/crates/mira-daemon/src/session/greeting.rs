//! Accueil d'une connexion : obtenir une place d'attente (en évinçant la plus ancienne si besoin), puis la poignée de main.

use std::time::Duration;

use tokio::io::{ReadHalf, WriteHalf};
use tokio::sync::OwnedSemaphorePermit;
use tokio::time::timeout;

use super::{Access, hello};
use crate::server::ServerContext;
use crate::transport::ByteStream;

const EVICTION_WAIT: Duration = Duration::from_secs(1);

type Reader = ReadHalf<Box<dyn ByteStream>>;
type Writer = WriteHalf<Box<dyn ByteStream>>;

/// Prend une place d'attente ; si elles sont toutes prises, la plus ancienne connexion en attente est évincée et sa place reprise.
async fn take_waiting_place(ctx: &ServerContext) -> Option<OwnedSemaphorePermit> {
    if let Ok(permit) = ctx.pending_hello.clone().try_acquire_owned() {
        return Some(permit);
    }
    ctx.pending.evict_oldest();
    let freed = ctx.pending_hello.clone().acquire_owned();
    timeout(EVICTION_WAIT, freed).await.ok()?.ok()
}

/// Accueille une connexion : sans place, elle est refusée ; évincée avant son `hello`, elle est abandonnée.
pub(super) async fn greet(
    reader: &mut Reader,
    writer: &mut Writer,
    ctx: &ServerContext,
) -> Option<Access> {
    let _place = take_waiting_place(ctx).await?;
    let entry = ctx.pending.register();
    tokio::select! {
        access = hello::handshake(reader, writer, ctx) => access,
        () = entry.evicted() => None,
    }
}
