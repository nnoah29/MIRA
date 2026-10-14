#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! La synchronisation respecte la politique réseau comme le rejeu et les lectures (serveur imposé par domaine).

mod support_sync;

use mira_core::sync::sync_account;
use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::{Security, ServerEndpoint, ServerSettings};
use support_sync::mirror::stored_messages;
use support_sync::provider::SyncTestProvider;
use support_sync::within;
use support_sync::world::{open_session, start_world};

/// Le serveur imposé pour le domaine de l'adresse l'emporte sur les serveurs mémorisés dans le compte
#[tokio::test]
async fn regression_sync_connection_honours_the_network_policy() {
    within(async {
        let mut world = start_world(CapabilitySet::full(), SyncTestProvider::folders()).await;
        let real = world.ctx.account.servers.clone().unwrap();
        let dead = ServerEndpoint {
            host: "127.0.0.1".to_string(),
            port: 1,
            security: Security::Plaintext,
        };
        world.ctx.account.servers = Some(ServerSettings {
            imap: dead.clone(),
            smtp: dead,
        });
        let overrides = &mut world.ctx.network.server_overrides;
        overrides.insert("example.org".to_string(), real);
        world.store.deliver(
            "alice",
            "INBOX",
            b"Subject: x\r\nMessage-ID: <x@x>\r\n\r\ncorps\r\n",
            &[],
        );
        let mut session = open_session(&world).await;
        sync_account(&mut session).await.unwrap();
        assert_eq!(stored_messages(&world.db), 1);
    })
    .await;
}
