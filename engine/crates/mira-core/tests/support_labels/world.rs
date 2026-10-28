//! Monde de synchronisation avec les dossiers usuels, et relevé des libellés système posés.

use mira_core::sync::{SyncSession, sync_account};
use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::{SystemLabelKind, ThreadId};

use crate::support_sync::provider::SyncTestProvider;
use crate::support_sync::world::{SyncWorld, open_session, start_world};

/// Monde avec Sent, Trash, Junk et Drafts créés, et une session ouverte (rien n'est encore synchronisé)
pub async fn open() -> (SyncWorld, SyncSession) {
    let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
    for name in ["Sent", "Trash", "Junk", "Drafts"] {
        world.store.create_mailbox("alice", name, None);
    }
    let session = open_session(&world).await;
    (world, session)
}

/// Une passe de synchronisation
pub async fn pass(session: &mut SyncSession) {
    sync_account(session).await.unwrap();
}

/// Sortes posées sur le fil de ce sujet (objet normalisé sans `Re:`), triées
pub fn kinds_of(world: &SyncWorld, subject: &str) -> Vec<SystemLabelKind> {
    let thread = thread_by_subject(world, subject);
    let listed = world
        .db
        .read(|tx| tx.list_system_label_assignments())
        .unwrap();
    let found = listed.into_iter().find(|a| a.thread == thread);
    found.map(|a| a.kinds).unwrap_or_default()
}

/// Fil dont le sujet est celui-ci
pub fn thread_by_subject(world: &SyncWorld, subject: &str) -> ThreadId {
    let account = crate::support_sync::mirror::account();
    let batch = world
        .db
        .read(|tx| tx.messages_for_threading(&account, None, 500))
        .unwrap();
    let found = batch
        .into_iter()
        .find(|(_, _, m)| m.subject.trim_start_matches("Re: ") == subject);
    found.unwrap().2.thread.unwrap()
}
