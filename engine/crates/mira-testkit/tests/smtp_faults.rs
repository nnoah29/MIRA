#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Pannes injectables du faux serveur SMTP : refus ciblés, coupure après DATA, arrêt.

mod smtp_support;

use std::time::Duration;

use smtp_support::LIMIT;
use smtp_support::client::SmtpClient;
use smtp_support::flow::{authenticated_client, send_message, started_server};
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout};

/// fail_next fait échouer le prochain MAIL FROM puis le suivant réussit.
#[tokio::test]
async fn fail_next_rejects_the_next_mail_command() {
    timeout(LIMIT, async {
        let (server, _) = started_server().await;
        let mut client = authenticated_client(&server).await;
        server.fail_next("452 4.3.1 Insufficient system storage");
        let reply = client.command("MAIL FROM:<alice@example.test>").await;
        assert_eq!(reply, "452 4.3.1 Insufficient system storage");
        let retry = client.command("MAIL FROM:<alice@example.test>").await;
        assert!(retry.starts_with("250"), "{retry}");
    })
    .await
    .unwrap();
}

/// fail_next fait échouer le prochain RCPT TO tel quel.
#[tokio::test]
async fn fail_next_rejects_the_next_recipient() {
    timeout(LIMIT, async {
        let (server, _) = started_server().await;
        let mut client = authenticated_client(&server).await;
        client.command("MAIL FROM:<alice@example.test>").await;
        server.fail_next("550 5.1.1 No such user");
        let reply = client.command("RCPT TO:<ghost@example.test>").await;
        assert_eq!(reply, "550 5.1.1 No such user");
    })
    .await
    .unwrap();
}

/// fail_next sur la fin de DATA refuse le message qui n'est pas enregistré.
#[tokio::test]
async fn fail_next_at_end_of_data_rejects_the_message() {
    timeout(LIMIT, async {
        let (server, _) = started_server().await;
        let mut client = authenticated_client(&server).await;
        let rcpts = ["bob@example.test"];
        client.command("MAIL FROM:<alice@example.test>").await;
        client.command("RCPT TO:<bob@example.test>").await;
        client.command("DATA").await;
        server.fail_next("552 5.3.4 Message too big");
        client.send_raw(b"Body\r\n.\r\n").await;
        assert_eq!(client.read_reply().await, "552 5.3.4 Message too big");
        assert!(server.received().is_empty());
        let reply = send_message(&mut client, "alice@example.test", &rcpts, "Body\r\n").await;
        assert!(reply.starts_with("250"), "{reply}");
    })
    .await
    .unwrap();
}

/// drop_after_data_once enregistre le message puis coupe sans répondre, une seule fois.
#[tokio::test]
async fn drop_after_data_once_records_then_cuts_without_reply() {
    timeout(LIMIT, async {
        let (server, _) = started_server().await;
        let rcpts = ["bob@example.test"];
        server.drop_after_data_once();
        let mut client = authenticated_client(&server).await;
        let reply = send_message(&mut client, "alice@example.test", &rcpts, "First\r\n").await;
        assert_eq!(reply, "");
        assert_eq!(server.received().len(), 1);
        let mut second = authenticated_client(&server).await;
        let reply = send_message(&mut second, "alice@example.test", &rcpts, "Second\r\n").await;
        assert!(reply.starts_with("250"), "{reply}");
        assert_eq!(server.received().len(), 2);
    })
    .await
    .unwrap();
}

/// Plusieurs pannes armées sont consommées dans l'ordre, une par commande.
#[tokio::test]
async fn armed_failures_are_consumed_in_order() {
    timeout(LIMIT, async {
        let (server, _) = started_server().await;
        let mut client = authenticated_client(&server).await;
        server.fail_next("451 4.7.1 Greylisted");
        server.fail_next("554 5.7.1 Rejected");
        let first = client.command("MAIL FROM:<alice@example.test>").await;
        let second = client.command("MAIL FROM:<alice@example.test>").await;
        assert_eq!(
            (first.as_str(), second.as_str()),
            ("451 4.7.1 Greylisted", "554 5.7.1 Rejected")
        );
    })
    .await
    .unwrap();
}

/// shutdown coupe les connexions ouvertes et refuse les nouvelles.
#[tokio::test]
async fn shutdown_cuts_connections_and_stops_listening() {
    timeout(LIMIT, async {
        let (server, _) = started_server().await;
        let mut client = SmtpClient::connect(&server.address()).await;
        server.shutdown();
        sleep(Duration::from_millis(100)).await;
        assert!(client.is_closed_by_peer().await);
        assert!(TcpStream::connect(server.address()).await.is_err());
    })
    .await
    .unwrap();
}
