#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Valeurs extrêmes du serveur : dates internes hors bornes, ensembles VANISHED énormes ou avec `*`.

mod support_imap;

use mira_core::imap::{ImapClient, ResyncFrom, SelectedMailbox};
use mira_vocabulary::Security;
use support_imap::env::local_endpoint;
use support_imap::logins::password_login;
use support_imap::scripted::scripted;
use support_imap::within;

const GREETING: &str = "* OK [CAPABILITY IMAP4rev1] pret\r\n";
const LOGGED_IN: &str = "{tag} OK [CAPABILITY IMAP4rev1 ENABLE CONDSTORE QRESYNC] connecte\r\n";
const SELECTED: &str = "{tag} OK [READ-WRITE] fait\r\n";

/// Date interne que le client lit dans une réponse FETCH portant `date` comme INTERNALDATE
async fn internal_date_of(date: &str) -> Option<i64> {
    let fetch =
        format!("* 1 FETCH (UID 1 RFC822.SIZE 9 INTERNALDATE \"{date}\")\r\n{{tag}} OK fait\r\n");
    let (port, task) = scripted(GREETING, &[LOGGED_IN, SELECTED, &fetch]).await;
    let target = local_endpoint(port, Security::Plaintext);
    let mut client = ImapClient::connect(&target, &password_login("secret"))
        .await
        .unwrap();
    client.select("INBOX", None).await.unwrap();
    let found = client.fetch_headers(&[1], &[]).await.unwrap();
    task.abort();
    found[0].internal_date.map(|date| date.as_millis())
}

/// Ouvre la boîte en QRESYNC face à un serveur qui annonce `vanished` comme disparus
async fn select_with_vanished(vanished: &str) -> SelectedMailbox {
    let enabled = "* ENABLED QRESYNC\r\n{tag} OK fait\r\n";
    let select = format!(
        "* 5 EXISTS\r\n* OK [UIDVALIDITY 7] x\r\n* OK [UIDNEXT 4000000000] x\r\n* VANISHED (EARLIER) {vanished}\r\n{{tag}} OK [READ-WRITE] fait\r\n"
    );
    let (port, task) = scripted(GREETING, &[LOGGED_IN, enabled, &select]).await;
    let target = local_endpoint(port, Security::Plaintext);
    let mut client = ImapClient::connect(&target, &password_login("secret"))
        .await
        .unwrap();
    assert!(client.enable_qresync().await.unwrap());
    let from = ResyncFrom {
        uid_validity: 7,
        highest_modseq: 1,
    };
    let selected = client.select("INBOX", Some(from)).await.unwrap();
    task.abort();
    selected
}

/// Année absurde (sans débordement), jour, heure, minute, fuseau et années hors bornes : la date est ignorée
#[tokio::test]
async fn regression_out_of_range_internal_dates_are_ignored() {
    within(async {
        let bad = [
            "01-Jan-922337203685477580 00:00:00 +0000",
            "31-Feb-2024 10:00:00 +0000",
            "01-Jan-1899 10:00:00 +0000",
            "01-Jan-10000 10:00:00 +0000",
            "01-Jan-2024 24:00:00 +0000",
            "01-Jan-2024 10:60:00 +0000",
            "01-Jan-2024 10:00:00 +9999",
        ];
        for date in bad {
            assert_eq!(internal_date_of(date).await, None, "{date}");
        }
    })
    .await;
}

/// Une date correcte garde sa valeur exacte, fuseau compris
#[tokio::test]
async fn valid_internal_date_is_converted_to_utc() {
    within(async {
        let found = internal_date_of("01-Jan-2024 10:00:00 +0200").await;
        assert_eq!(found, Some(1_704_096_000_000));
    })
    .await;
}

/// Un ensemble VANISHED de plusieurs millions d'UID n'est pas tronqué en silence : il est signalé incomplet
#[tokio::test]
async fn regression_huge_vanished_set_is_flagged_incomplete() {
    within(async {
        let selected = select_with_vanished("1:3000000").await;
        assert!(selected.vanished_incomplete);
        assert!(selected.vanished.len() <= 2_000_000);
    })
    .await;
}

/// Un ensemble VANISHED avec `*` ne peut pas être développé : il est signalé incomplet, jamais ignoré
#[tokio::test]
async fn regression_vanished_set_with_star_is_flagged_incomplete() {
    within(async {
        let selected = select_with_vanished("3,5:*").await;
        assert!(selected.vanished_incomplete);
        assert!(selected.vanished.contains(&3));
    })
    .await;
}
