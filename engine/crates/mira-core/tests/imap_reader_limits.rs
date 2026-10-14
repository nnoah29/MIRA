#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Lecteur IMAP face à un serveur hostile ou lent : mémoire bornée, délai d'inactivité réarmé à chaque bloc.

mod support_imap;

use std::time::Duration;

use mira_core::imap::{ImapClient, ImapOptions};
use mira_vocabulary::{MiraError, Security};
use support_imap::env::local_endpoint;
use support_imap::hostile::{Step, hostile_server};
use support_imap::logins::password_login;
use support_imap::within;

/// Se connecte au serveur hostile, sélectionne la boîte puis demande la source du message 1
async fn fetch_from(steps: Vec<Step>, timeout: Duration) -> Result<Option<Vec<u8>>, MiraError> {
    let (port, task) = hostile_server(steps).await;
    let target = local_endpoint(port, Security::Plaintext);
    let options = ImapOptions { timeout };
    let mut client = ImapClient::connect_with(&target, &password_login("secret"), options).await?;
    client.select("INBOX", None).await?;
    let outcome = client.fetch_source(1).await;
    task.abort();
    outcome
}

/// Une ligne sans fin de plus de 1 Mo est refusée au lieu d'être accumulée
#[tokio::test]
async fn regression_endless_line_is_refused() {
    within(async {
        let mut line = b"* ".to_vec();
        line.extend(std::iter::repeat_n(b'A', 2 * 1024 * 1024));
        let steps = vec![Step::Send(line), Step::Pause(Duration::from_secs(30))];
        let error = fetch_from(steps, Duration::from_secs(10))
            .await
            .unwrap_err();
        assert_eq!(error.code, "server.protocol");
    })
    .await;
}

/// Un littéral annoncé au-delà du plafond est refusé avant d'être alloué ou lu
#[tokio::test]
async fn regression_huge_literal_announcement_is_refused() {
    within(async {
        let head = b"* 1 FETCH (UID 1 BODY[] {100000000}\r\n".to_vec();
        let steps = vec![Step::Send(head), Step::Pause(Duration::from_secs(30))];
        let error = fetch_from(steps, Duration::from_secs(10))
            .await
            .unwrap_err();
        assert_eq!(error.code, "server.protocol");
    })
    .await;
}

/// Le cumul des lignes d'une même réponse est plafonné
#[tokio::test]
async fn regression_unbounded_response_is_refused() {
    within(async {
        let mut line = b"* ".to_vec();
        line.extend(std::iter::repeat_n(b'B', 900 * 1024));
        line.extend_from_slice(b"\r\n");
        let mut steps = vec![Step::Send(line.clone()); 80];
        steps.push(Step::Send(b"{tag} OK fini\r\n".to_vec()));
        let error = fetch_from(steps, Duration::from_secs(10))
            .await
            .unwrap_err();
        assert_eq!(error.code, "server.protocol");
    })
    .await;
}

/// Un gros littéral lent mais régulier n'expire pas : le délai d'inactivité est réarmé à chaque bloc
#[tokio::test]
async fn regression_slow_literal_survives_while_bytes_keep_arriving() {
    within(async {
        let mut steps = vec![Step::Send(b"* 1 FETCH (UID 1 BODY[] {600}\r\n".to_vec())];
        for _ in 0..6 {
            steps.push(Step::Send(vec![b'x'; 100]));
            steps.push(Step::Pause(Duration::from_millis(200)));
        }
        steps.push(Step::Send(b")\r\n{tag} OK fait\r\n".to_vec()));
        let source = fetch_from(steps, Duration::from_millis(500)).await;
        assert_eq!(source.unwrap().unwrap().len(), 600);
    })
    .await;
}

/// Un littéral qui s'arrête en route expire après un seul délai d'inactivité
#[tokio::test]
async fn stalled_literal_times_out_after_one_idle_period() {
    within(async {
        let steps = vec![
            Step::Send(b"* 1 FETCH (UID 1 BODY[] {600}\r\n".to_vec()),
            Step::Send(vec![b'x'; 100]),
            Step::Pause(Duration::from_secs(30)),
        ];
        let error = fetch_from(steps, Duration::from_millis(300))
            .await
            .unwrap_err();
        assert_eq!(error.code, "network.timeout");
    })
    .await;
}
