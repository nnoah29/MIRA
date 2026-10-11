#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Recherche d'UID et récupération des en-têtes, des drapeaux et de la source.

mod support_imap;

use mira_core::imap::ImapClient;
use mira_core::mime::parse_headers;
use mira_testkit::imap::CapabilitySet;
use mira_vocabulary::Timestamp;
use support_imap::env::start;
use support_imap::logins::selected;
use support_imap::mail::{MAIL, mail, now_epoch};
use support_imap::within;

const DAY: i64 = 86_400;

/// `uids_since` ne rend que les messages assez récents, `all_uids` les rend tous
#[tokio::test]
async fn uids_since_and_all_uids() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let now = now_epoch();
        for age in [2, 10, 60] {
            let raw = mail("s", "a@b");
            env.store
                .deliver_dated("alice", "INBOX", &raw, &[], now - age * DAY);
        }
        let mut client = selected(&env).await;
        let since = Timestamp::from_millis((now - 30 * DAY) * 1000);
        assert_eq!(client.uids_since(since).await.unwrap(), vec![1, 2]);
        assert_eq!(client.all_uids().await.unwrap(), vec![1, 2, 3]);
    })
    .await;
}

/// Les en-têtes bruts, drapeaux, taille et date interne sont rendus et lisibles par le module MIME
#[tokio::test]
async fn fetch_headers_gives_raw_headers_and_metadata() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store
            .deliver_dated("alice", "INBOX", MAIL, &["\\Seen"], 1_700_000_000);
        let mut client = selected(&env).await;
        let fetched = client.fetch_headers(&[1], &[]).await.unwrap();
        let first = &fetched[0];
        assert_eq!((first.uid, first.size), (1, MAIL.len() as u64));
        assert_eq!(first.flags, vec!["\\Seen"]);
        assert_eq!(first.internal_date.unwrap().as_millis(), 1_700_000_000_000);
        assert!(first.modseq.is_some() && first.extra.items.is_empty());
        assert_eq!(parse_headers(&first.header_bytes).subject, "Hello");
        assert!(!String::from_utf8_lossy(&first.header_bytes).contains("Body text"));
    })
    .await;
}

/// Les attributs propres au service sont rendus bruts, ou refusés si l'extension manque
#[tokio::test]
async fn fetch_headers_with_extra_attributes() {
    within(async {
        let caps = CapabilitySet::full().with_label_extensions();
        let env = start(caps).await;
        env.store
            .deliver_with_labels("alice", &["INBOX", "Work"], MAIL, 777, 888);
        let mut client = selected(&env).await;
        let extra = ["X-GM-MSGID", "X-GM-THRID", "x-gm-labels"].map(String::from);
        let fetched = client.fetch_headers(&[1], &extra).await.unwrap();
        let items = &fetched[0].extra.items;
        assert_eq!(items[0], ("X-GM-MSGID".to_string(), "777".to_string()));
        assert_eq!(items[1], ("X-GM-THRID".to_string(), "888".to_string()));
        assert!(
            items[2].1.contains("Work") && items[2].1.contains("\\Inbox"),
            "{items:?}"
        );
        let plain = start(CapabilitySet::full()).await;
        plain.store.deliver("alice", "INBOX", MAIL, &[]);
        let mut bare = selected(&plain).await;
        let error = bare.fetch_headers(&[1], &extra).await.unwrap_err();
        assert_eq!(error.code, "server.extension_missing");
    })
    .await;
}

/// La source complète est rendue sans poser `\Seen`, y compris quand elle imite du protocole
#[tokio::test]
async fn fetch_source_is_complete_and_does_not_mark_seen() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let tricky = b"Subject: piege\r\n\r\n)\r\n* 1 FETCH (UID 9 FLAGS (\\Seen))\r\n{12}\r\nA1 OK done\r\n";
        env.store.deliver("alice", "INBOX", tricky, &[]);
        let mut client = selected(&env).await;
        assert_eq!(client.fetch_source(1).await.unwrap().unwrap(), tricky);
        assert_eq!(client.fetch_source(99).await.unwrap(), None);
        assert!(env.store.messages("alice", "INBOX")[0].flags.is_empty());
    })
    .await;
}

/// Les drapeaux sont lus, et CHANGEDSINCE ne rend que les messages modifiés
#[tokio::test]
async fn fetch_flags_with_changed_since() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        for _ in 0..3 {
            env.store.deliver("alice", "INBOX", MAIL, &[]);
        }
        let baseline = env.store.highest_modseq("alice", "INBOX").unwrap();
        env.store
            .set_flags("alice", "INBOX", 2, &["\\Seen", "\\Flagged"]);
        let mut client = selected(&env).await;
        let all = client.fetch_flags(&[1, 2, 3], None).await.unwrap();
        assert_eq!(all.len(), 3);
        let changed = client
            .fetch_flags(&[1, 2, 3], Some(baseline))
            .await
            .unwrap();
        assert_eq!(changed.len(), 1);
        assert_eq!(changed[0].uid, 2);
        let mut flags = changed[0].flags.clone();
        flags.sort();
        assert_eq!(flags, vec!["\\Flagged", "\\Seen"]);
        assert!(changed[0].modseq.unwrap() > baseline);
    })
    .await;
}

/// Sans boîte sélectionnée l'opération échoue localement, et les longues listes d'UID sont découpées
#[tokio::test]
async fn selection_is_required_and_long_lists_are_split() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        for _ in 0..1200 {
            env.store.deliver("alice", "INBOX", MAIL, &[]);
        }
        let bare = ImapClient::connect(
            &support_imap::env::endpoint(&env),
            &support_imap::logins::password_login("secret"),
        )
        .await;
        let error = bare.unwrap().all_uids().await.unwrap_err();
        assert_eq!(error.code, "internal.bug");
        let mut client = selected(&env).await;
        let uids = client.all_uids().await.unwrap();
        let fetched = client.fetch_headers(&uids, &[]).await.unwrap();
        assert_eq!(fetched.len(), 1200);
        let log = env.store.command_log();
        assert!(
            log.iter().filter(|l| l.contains("FETCH")).count() >= 3,
            "{log:?}"
        );
    })
    .await;
}
