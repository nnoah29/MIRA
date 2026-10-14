//! Actions locales mises en file pendant un test de synchronisation.

use mira_core::actions::{QueueContext, SubmitRequest, submit};
use mira_vocabulary::{ActionKind, IdempotencyKey, MessageId};

use super::mirror::account;
use super::world::SyncWorld;

/// Met en file « marquer lu » pour un message : son effet local est appliqué tout de suite
pub fn queue_mark_read(world: &SyncWorld, target: &MessageId) {
    let ctx = &world.ctx;
    let queue = QueueContext {
        db: &world.db,
        clock: &*ctx.clock,
        ids: &*ctx.ids,
        events: &*ctx.events,
    };
    let request = SubmitRequest {
        account: account(),
        kind: ActionKind::MarkRead { read: true },
        targets: vec![target.clone()],
        key: IdempotencyKey("k1".to_string()),
    };
    submit(&queue, &ctx.provider.descriptor().quirks, request).unwrap();
}
