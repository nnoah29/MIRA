//! Scénarios réutilisés par plusieurs tests : changements du serveur, coupure puis reprise, déplacement.

use std::sync::Arc;
use std::time::Duration;

use mira_core::events::RecordingSink;
use mira_core::operations::{OperationRegistry, OperationStatus};
use mira_core::sync::{start_extend_window, sync_account};
use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::{EventKind, LogLevel, MiraError, SyncMarker};

use super::cutting::CuttingSink;
use super::mail::{days_ago, mail};
use super::mirror::{mailbox_named, mirror_rows};
use super::provider::SyncTestProvider;
use super::supervised::start_supervised;
use super::waiting::{finished_passes, wait_until};
use super::world::{SyncWorld, build_world, deliver_many, open_session, start_server, start_world};

/// Quatre messages, première synchro, puis drapeau lu, suppression, nouveau message, seconde synchro
pub async fn changes_scenario(caps: CapabilitySet) -> SyncWorld {
    let world = start_world(caps, SyncTestProvider::folders()).await;
    deliver_many(&world, "INBOX", "Courrier", 4);
    let mut session = open_session(&world).await;
    sync_account(&mut session).await.unwrap();
    world.store.set_flags("alice", "INBOX", 1, &["\\Seen"]);
    world.store.expunge("alice", "INBOX", 2);
    let fifth = mail("Courrier 5", "s5@x");
    world.store.deliver("alice", "INBOX", &fifth, &[]);
    sync_account(&mut session).await.unwrap();
    world
}

/// Monde dont le serveur est coupé après le premier lot d'en-têtes intégré
async fn world_cut_after_first_batch(caps: CapabilitySet) -> SyncWorld {
    let (server, store) = start_server(caps).await;
    let recording = Arc::new(RecordingSink::default());
    let cutting = CuttingSink::new(Arc::clone(&recording), Arc::clone(&server));
    let provider = SyncTestProvider::folders();
    build_world(server, store, provider, recording, Arc::new(cutting))
}

/// Résultat d'une synchronisation coupée au milieu puis reprise.
pub struct CutRun {
    pub world: SyncWorld,
    pub failure: MiraError,
    pub rows_after_cut: usize,
    pub marker_after_cut: Option<SyncMarker>,
}

/// Synchronise `count` messages en coupant le serveur après le premier lot d'en-têtes, puis reprend
pub async fn cut_then_resume(caps: CapabilitySet, count: u32) -> CutRun {
    let world = world_cut_after_first_batch(caps).await;
    deliver_many(&world, "INBOX", "Lot", count);
    let mut session = open_session(&world).await;
    let failure = sync_account(&mut session).await.unwrap_err();
    let rows_after_cut = mirror_rows(&world.db).len();
    let marker_after_cut = mailbox_named(&world.db, "INBOX").marker;
    let mut resumed = open_session(&world).await;
    sync_account(&mut resumed).await.unwrap();
    CutRun {
        world,
        failure,
        rows_after_cut,
        marker_after_cut,
    }
}

/// Déplace un message de la réception vers « Archive » depuis un autre appareil (commande MOVE)
pub async fn move_to_archive(world: &SyncWorld, uid: u32) {
    let mut other = open_session(world).await;
    other.conn.client.select("INBOX", None).await.unwrap();
    other
        .conn
        .client
        .move_messages(&[uid], "Archive")
        .await
        .unwrap();
}

/// Relevés d'un compte très sollicité : pic de connexions ouvertes et total de connexions acceptées.
pub struct BusyRun {
    pub peak: usize,
    pub total: usize,
    pub world: SyncWorld,
}

/// Compte supervisé qui reçoit des demandes en rafale pendant une extension de plage, sous une limite de connexions
pub async fn busy_account(limit: u32) -> BusyRun {
    let provider = SyncTestProvider::folders().max_connections(limit);
    let world = start_world(CapabilitySet::full(), provider).await;
    world
        .store
        .deliver_dated("alice", "INBOX", &mail("Vieux", "v@x"), &[], days_ago(100));
    world
        .store
        .deliver("alice", "INBOX", &mail("Récent", "r@x"), &[]);
    let supervisor = start_supervised(&world, Duration::from_millis(100));
    wait_until(|| finished_passes(&world.recording) >= 1).await;
    let budget = supervisor.budget_of(&world.ctx.account.id).unwrap();
    let registry = OperationRegistry::new(world.ctx.ids.clone(), world.ctx.events.clone());
    let inbox = mailbox_named(&world.db, "INBOX").id;
    let (ctx, credentials) = (world.ctx.clone(), world.credentials.clone());
    let operation = start_extend_window(ctx, budget, credentials, &registry, inbox, 30);
    let mut peak = 0;
    while registry.get(&operation).unwrap().status != OperationStatus::Finished {
        peak = peak.max(world.server.active_connections());
        supervisor.request_sync(&world.ctx.account.id);
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(supervisor.shutdown(Duration::from_secs(5)).await);
    let total = world.server.connection_count();
    BusyRun { peak, total, world }
}

/// Textes des événements de diagnostic de décision de stratégie
pub fn strategy_notes(world: &SyncWorld) -> Vec<String> {
    world
        .recording
        .kinds()
        .into_iter()
        .filter_map(|kind| match kind {
            EventKind::Diagnostic {
                level: LogLevel::Info,
                target,
                text,
            } if target == "sync.strategy" => Some(text),
            _ => None,
        })
        .collect()
}
