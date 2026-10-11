#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! CONDSTORE, QRESYNC, VANISHED et renumérotation du faux serveur IMAP.

mod imap_support;

use imap_support::fixtures::{MAIL, logged_in, selected, start};
use imap_support::within;
use mira_testkit::imap::CapabilitySet;

/// ENABLE QRESYNC est confirmé par ENABLED
#[tokio::test]
async fn enable_qresync_is_acknowledged() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = logged_in(&env).await;
        assert!(
            client
                .command("ENABLE QRESYNC")
                .await
                .has("ENABLED QRESYNC")
        );
    })
    .await;
}

/// SELECT avec QRESYNC renvoie VANISHED (EARLIER) et les changements de drapeaux
#[tokio::test]
async fn select_qresync_returns_vanished_and_changes() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        for _ in 0..3 {
            env.store.deliver("alice", "INBOX", MAIL, &[]);
        }
        let baseline = env.store.highest_modseq("alice", "INBOX").unwrap();
        let validity = env.store.uidvalidity("alice", "INBOX").unwrap();
        env.store.set_flags("alice", "INBOX", 1, &["\\Seen"]);
        env.store.expunge("alice", "INBOX", 3);
        let mut client = logged_in(&env).await;
        client.command("ENABLE QRESYNC").await;
        let response = client
            .command(&format!("SELECT INBOX (QRESYNC ({validity} {baseline}))"))
            .await;
        assert!(response.has("VANISHED (EARLIER) 3"), "{response:?}");
        assert!(response.has("1 FETCH (UID 1 FLAGS (\\Seen) MODSEQ ("));
        assert!(response.has("2 EXISTS"));
    })
    .await;
}

/// QRESYNC sans ENABLE préalable est refusé
#[tokio::test]
async fn select_qresync_requires_enable() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = logged_in(&env).await;
        let response = client.command("SELECT INBOX (QRESYNC (1 1))").await;
        assert!(response.status.contains(" BAD "), "{response:?}");
    })
    .await;
}

/// FETCH CHANGEDSINCE VANISHED annonce les disparitions et les changements
#[tokio::test]
async fn fetch_changedsince_with_vanished() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        for _ in 0..3 {
            env.store.deliver("alice", "INBOX", MAIL, &[]);
        }
        let mut client = logged_in(&env).await;
        client.command("ENABLE QRESYNC").await;
        client.command("SELECT INBOX").await;
        let baseline = env.store.highest_modseq("alice", "INBOX").unwrap();
        env.store.set_flags("alice", "INBOX", 2, &["\\Seen"]);
        env.store.expunge("alice", "INBOX", 1);
        let request = format!("UID FETCH 1:* (FLAGS) (CHANGEDSINCE {baseline} VANISHED)");
        let response = client.command(&request).await;
        assert!(response.has("VANISHED (EARLIER) 1"), "{response:?}");
        assert!(response.has("UID 2 FLAGS (\\Seen) MODSEQ ("));
        assert_eq!(response.count("FETCH"), 1);
    })
    .await;
}

/// La renumérotation change UIDVALIDITY et réattribue les UID
#[tokio::test]
async fn renumber_changes_uidvalidity() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        env.store.expunge("alice", "INBOX", 1);
        let before = env.store.uidvalidity("alice", "INBOX").unwrap();
        env.store.renumber("alice", "INBOX");
        let after = env.store.uidvalidity("alice", "INBOX").unwrap();
        assert_ne!(before, after);
        let mut client = logged_in(&env).await;
        let response = client.command("SELECT INBOX").await;
        assert!(response.has(&format!("[UIDVALIDITY {after}]")) && response.has("[UIDNEXT 2]"));
        assert_eq!(env.store.messages("alice", "INBOX")[0].uid, 1);
    })
    .await;
}

/// Sans QRESYNC, l'extension n'est ni annoncée ni activable
#[tokio::test]
async fn qresync_absent_is_not_advertised() {
    within(async {
        let env = start(CapabilitySet::without_qresync()).await;
        let mut client = selected(&env).await;
        assert!(!client.command("CAPABILITY").await.has("QRESYNC"));
        assert!(!client.command("ENABLE QRESYNC").await.has("ENABLED"));
        let reselect = client.command("SELECT INBOX (QRESYNC (1 1))").await;
        assert!(reselect.status.contains(" BAD "), "{reselect:?}");
    })
    .await;
}
