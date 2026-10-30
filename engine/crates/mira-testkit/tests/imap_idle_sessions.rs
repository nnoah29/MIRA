#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Nombre de sessions IDLE réellement à l'écoute, pour synchroniser les tests sur la fin du démarrage.

mod imap_support;

use std::time::Duration;

use imap_support::fixtures::{selected, start};
use imap_support::within;
use mira_testkit::imap::{CapabilitySet, FakeImapServer};

/// Attend que le serveur compte exactement ce nombre de sessions IDLE
async fn wait_idle_sessions(server: &FakeImapServer, expected: usize) {
    while server.idle_sessions() != expected {
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
}

/// Une session compte comme à l'écoute une fois l'IDLE établi, et plus après DONE
#[tokio::test]
async fn idle_session_is_counted_while_listening() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = selected(&env).await;
        assert_eq!(env.server.idle_sessions(), 0);
        client.send_raw("i1 IDLE\r\n").await;
        wait_idle_sessions(&env.server, 1).await;
        env.store
            .deliver("alice", "INBOX", imap_support::fixtures::MAIL, &[]);
        assert_eq!(client.read_until_contains("EXISTS").await, "* 1 EXISTS");
        client.send_raw("DONE\r\n").await;
        assert!(client.read_response("i1").await.is_ok());
        wait_idle_sessions(&env.server, 0).await;
    })
    .await;
}
