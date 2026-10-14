#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Descente d'un compte (S4) : boîtes, en-têtes de la réception d'abord, corps préchargés, progression.

mod support_sync;

use mira_core::reading::get_body;
use mira_core::sync::sync_account;
use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::{ContentState, EventKind, MailboxRole, PageRequest, SyncStep};
use support_sync::events::{announced, has_progress_step, header_progress};
use support_sync::inspect::{inbox_page, set_prefetch};
use support_sync::mail::{mail, reply};
use support_sync::mirror::{account, mailbox_named, mirror_rows};
use support_sync::provider::SyncTestProvider;
use support_sync::waiting::count_events;
use support_sync::within;
use support_sync::world::{deliver_many, open_session, start_world};

/// S4 : la première synchronisation télécharge la réception d'abord, avec progression et sans événement « nouveau message »
#[tokio::test]
async fn scenario_s4_first_sync_downloads_inbox_first_with_progress() {
    within(async {
        let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
        world.store.create_mailbox("alice", "Sent", Some("\\Sent"));
        let sent = mail("Envoyé", "s1@x");
        world.store.deliver("alice", "Sent", &sent, &["\\Seen"]);
        deliver_many(&world, "INBOX", "Reçu", 3);
        let mut session = open_session(&world).await;
        sync_account(&mut session).await.unwrap();
        let kinds = world.recording.kinds();
        assert!(matches!(kinds.first(), Some(EventKind::SyncStarted { .. })));
        assert!(matches!(kinds.last(), Some(EventKind::SyncFinished { .. })));
        let headers = header_progress(&world.recording);
        assert_eq!(headers.first(), Some(&(0, 3)), "la réception vient d'abord");
        assert!(headers.contains(&(3, 3)) && headers.contains(&(1, 1)));
        assert!(has_progress_step(&world.recording, SyncStep::Mailboxes));
        assert!(has_progress_step(&world.recording, SyncStep::Bodies));
        assert_eq!(announced(&world.recording), 0);
        assert_eq!(mirror_rows(&world.db).len(), 4);
    })
    .await;
}

/// Les en-têtes créent messages, drapeaux du serveur, fils et index de recherche
#[tokio::test]
async fn first_sync_stores_flags_threads_and_search_index() {
    within(async {
        let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
        let root = mail("Projet lune", "root@x");
        let answer = reply("Projet lune", "ans@x", "root@x");
        world
            .store
            .deliver("alice", "INBOX", &root, &["\\Seen", "\\Flagged"]);
        world.store.deliver("alice", "INBOX", &answer, &[]);
        let mut session = open_session(&world).await;
        sync_account(&mut session).await.unwrap();
        let rows = mirror_rows(&world.db);
        assert!(rows[0].seen && rows[0].flagged && !rows[1].seen);
        let found = world
            .db
            .read(|tx| tx.search_messages(Some(&account()), "lune", &PageRequest::default()));
        let page = found.unwrap();
        assert_eq!(page.items.len(), 2);
        assert_eq!(page.items[0].thread, page.items[1].thread);
        assert!(page.items[0].thread.is_some());
        let inbox = mailbox_named(&world.db, "INBOX");
        assert_eq!(
            (inbox.total, inbox.unread, inbox.role),
            (2, 1, MailboxRole::Inbox)
        );
    })
    .await;
}

/// Les corps des messages récents de la réception sont préchargés : source, texte, aperçu, index, événement
#[tokio::test]
async fn first_sync_prefetches_bodies_of_the_most_recent_inbox_messages() {
    within(async {
        let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
        set_prefetch(&world.db, 2);
        deliver_many(&world, "INBOX", "Mensuel", 3);
        let mut session = open_session(&world).await;
        sync_account(&mut session).await.unwrap();
        let page = inbox_page(&world.db);
        let with_body: Vec<_> = page
            .items
            .iter()
            .filter(|m| m.content == ContentState::BodyPresent)
            .collect();
        assert_eq!(with_body.len(), 2);
        assert!(with_body[0].preview.starts_with("Corps de Mensuel"));
        assert!(
            get_body(&world.db, &with_body[0].id)
                .unwrap()
                .text
                .contains("Corps de")
        );
        let available = count_events(&world.recording, |k| {
            matches!(k, EventKind::BodyAvailable { .. })
        });
        assert_eq!(available, 2);
    })
    .await;
}

/// Une boîte supprimée sur le serveur disparaît du miroir avec ses messages (le serveur gagne)
#[tokio::test]
async fn mailbox_deleted_on_server_leaves_the_mirror() {
    within(async {
        let world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
        world
            .store
            .deliver("alice", "INBOX", &mail("Reste", "r@x"), &[]);
        world
            .store
            .deliver("alice", "Vieux", &mail("Part", "p@x"), &[]);
        let mut session = open_session(&world).await;
        sync_account(&mut session).await.unwrap();
        assert_eq!(mirror_rows(&world.db).len(), 2);
        let mut other = open_session(&world).await;
        other.conn.client.delete_mailbox("Vieux").await.unwrap();
        sync_account(&mut session).await.unwrap();
        let rows = mirror_rows(&world.db);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].subject, "Reste");
        let gone = count_events(&world.recording, |k| {
            matches!(k, EventKind::MailboxDeleted { .. })
        });
        assert_eq!(gone, 1);
    })
    .await;
}
