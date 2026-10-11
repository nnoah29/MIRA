#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Resynchronisation rapide : ENABLE QRESYNC, VANISHED et changements de drapeaux.

mod support_imap;

use mira_core::imap::ResyncFrom;
use mira_testkit::imap::CapabilitySet;
use support_imap::env::{Env, start};
use support_imap::logins::connected;
use support_imap::mail::MAIL;
use support_imap::within;

/// Prépare trois messages, puis un changement de drapeaux et une suppression depuis un autre appareil
async fn env_with_changes() -> (Env, ResyncFrom) {
    let env = start(CapabilitySet::full()).await;
    for _ in 0..3 {
        env.store.deliver("alice", "INBOX", MAIL, &[]);
    }
    let from = ResyncFrom {
        uid_validity: env.store.uidvalidity("alice", "INBOX").unwrap(),
        highest_modseq: env.store.highest_modseq("alice", "INBOX").unwrap(),
    };
    env.store.set_flags("alice", "INBOX", 1, &["\\Seen"]);
    env.store.expunge("alice", "INBOX", 3);
    (env, from)
}

/// ENABLE QRESYNC est accepté par un serveur complet et ignoré par un serveur sans l'extension
#[tokio::test]
async fn enable_qresync_depends_on_server_support() {
    within(async {
        let full = start(CapabilitySet::full()).await;
        assert!(connected(&full).await.enable_qresync().await.unwrap());
        let without = start(CapabilitySet::without_qresync()).await;
        assert!(!connected(&without).await.enable_qresync().await.unwrap());
        let minimal = start(CapabilitySet::minimal()).await;
        assert!(!connected(&minimal).await.enable_qresync().await.unwrap());
    })
    .await;
}

/// SELECT avec resynchronisation rend les UID disparus et les drapeaux modifiés
#[tokio::test]
async fn select_with_resync_returns_vanished_and_changes() {
    within(async {
        let (env, from) = env_with_changes().await;
        let mut client = connected(&env).await;
        assert!(client.enable_qresync().await.unwrap());
        let selected = client.select("INBOX", Some(from)).await.unwrap();
        assert_eq!(selected.vanished, vec![3]);
        assert_eq!(selected.changed.len(), 1);
        assert_eq!(selected.changed[0].uid, 1);
        assert_eq!(selected.changed[0].flags, vec!["\\Seen"]);
        assert!(selected.changed[0].modseq.unwrap() > from.highest_modseq);
        assert_eq!(selected.exists, 2);
        assert_eq!(
            selected.highest_modseq,
            env.store.highest_modseq("alice", "INBOX")
        );
    })
    .await;
}

/// Demander une resynchronisation sans avoir activé QRESYNC est refusé localement
#[tokio::test]
async fn resync_without_enable_is_refused() {
    within(async {
        let (env, from) = env_with_changes().await;
        let mut client = connected(&env).await;
        let error = client.select("INBOX", Some(from)).await.unwrap_err();
        assert_eq!(error.code, "server.extension_missing");
        let plain = client.select("INBOX", None).await.unwrap();
        assert!(plain.vanished.is_empty() && plain.changed.is_empty());
    })
    .await;
}

/// Après une renumérotation, UIDVALIDITY change et aucune différence n'est annoncée
#[tokio::test]
async fn changed_uid_validity_is_visible_to_the_caller() {
    within(async {
        let (env, from) = env_with_changes().await;
        env.store.renumber("alice", "INBOX");
        let mut client = connected(&env).await;
        client.enable_qresync().await.unwrap();
        let selected = client.select("INBOX", Some(from)).await.unwrap();
        assert_ne!(selected.uid_validity, from.uid_validity);
        assert!(selected.vanished.is_empty() && selected.changed.is_empty());
        assert_eq!(selected.uid_next, 3);
    })
    .await;
}

/// Une plage d'UID disparus est étendue en liste
#[tokio::test]
async fn vanished_ranges_are_expanded() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        for _ in 0..6 {
            env.store.deliver("alice", "INBOX", MAIL, &[]);
        }
        let from = ResyncFrom {
            uid_validity: env.store.uidvalidity("alice", "INBOX").unwrap(),
            highest_modseq: env.store.highest_modseq("alice", "INBOX").unwrap(),
        };
        for uid in [2, 3, 4, 6] {
            env.store.expunge("alice", "INBOX", uid);
        }
        let mut client = connected(&env).await;
        client.enable_qresync().await.unwrap();
        let selected = client.select("INBOX", Some(from)).await.unwrap();
        assert_eq!(selected.vanished, vec![2, 3, 4, 6]);
        assert_eq!(selected.exists, 2);
    })
    .await;
}
