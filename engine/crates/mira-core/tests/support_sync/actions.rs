//! Actions locales mises en file pendant un test de synchronisation.

use mira_core::actions::{QueueContext, SubmitRequest, submit};
use mira_vocabulary::{ActionKind, IdempotencyKey, MailboxId, MessageId};

use super::mirror::account;
use super::world::SyncWorld;

/// Met en file une action sur un message : son effet local est appliqué tout de suite
pub fn queue(world: &SyncWorld, kind: ActionKind, target: &MessageId, key: &str) {
    let ctx = &world.ctx;
    let queue = QueueContext {
        db: &world.db,
        clock: &*ctx.clock,
        ids: &*ctx.ids,
        events: &*ctx.events,
    };
    let request = SubmitRequest {
        account: account(),
        kind,
        targets: vec![target.clone()],
        key: IdempotencyKey(key.to_string()),
    };
    submit(&queue, &ctx.provider.descriptor().quirks, request).unwrap();
}

/// Met en file « marquer lu » pour un message
pub fn queue_mark_read(world: &SyncWorld, target: &MessageId) {
    queue(world, ActionKind::MarkRead { read: true }, target, "k-read");
}

/// Met en file le déplacement d'un message vers une boîte
pub fn queue_move(world: &SyncWorld, target: &MessageId, to: &MailboxId) {
    queue(world, ActionKind::Move { to: to.clone() }, target, "k-move");
}
