#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Serveur de test et moteur de synchronisation réels, pilotés par une suite de changements aléatoires.

mod changes;
pub mod lookup;
mod observe;
pub mod ops;

use std::collections::HashMap;

use mira_core::sync::sync_account;
use mira_testkit::imap::CapabilitySet;

use crate::support_sync::provider::SyncTestProvider;
use crate::support_sync::within;
use crate::support_sync::world::{open_session, start_world};
use ops::ServerOp;

/// Rejoue les changements sur un faux serveur à libellés ou à dossiers ; après chaque synchronisation, le miroir est vérifié
pub fn run_server_sequence(operations: &[ServerOp], stable_ids: bool) {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(within(replay(operations, stable_ids)));
}

/// Corps asynchrone de la séquence ; une dernière synchronisation termine toujours la suite
async fn replay(operations: &[ServerOp], stable_ids: bool) {
    let caps = CapabilitySet::full().with_label_extensions();
    let provider = if stable_ids {
        SyncTestProvider::with_stable_ids()
    } else {
        SyncTestProvider::folders()
    };
    let world = start_world(caps, provider).await;
    let mut session = open_session(&world).await;
    let mut known = HashMap::new();
    let closing = [ServerOp::Sync];
    for operation in operations.iter().chain(closing.iter()) {
        match operation {
            ServerOp::Sync => {
                sync_account(&mut session).await.unwrap();
                observe::check_after_sync(&world.db, &world.store, &mut known);
            }
            other => changes::perform(&world.store, other),
        }
    }
}
