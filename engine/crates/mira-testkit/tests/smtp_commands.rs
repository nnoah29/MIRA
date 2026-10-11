#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Commandes SMTP annexes : RSET, NOOP, QUIT, séquences invalides.

mod smtp_support;

use smtp_support::LIMIT;
use smtp_support::flow::{authenticated_client, started_server};
use tokio::time::timeout;

/// RSET abandonne l'enveloppe : RCPT qui suit est hors séquence.
#[tokio::test]
async fn rset_clears_the_envelope() {
    timeout(LIMIT, async {
        let (server, _) = started_server().await;
        let mut client = authenticated_client(&server).await;
        client.command("MAIL FROM:<alice@example.test>").await;
        assert!(client.command("RSET").await.starts_with("250"));
        let reply = client.command("RCPT TO:<bob@example.test>").await;
        assert!(reply.starts_with("503"), "{reply}");
    })
    .await
    .unwrap();
}

/// DATA sans destinataire est refusé par 503.
#[tokio::test]
async fn data_without_recipient_is_refused() {
    timeout(LIMIT, async {
        let (server, _) = started_server().await;
        let mut client = authenticated_client(&server).await;
        client.command("MAIL FROM:<alice@example.test>").await;
        assert!(client.command("DATA").await.starts_with("503"));
    })
    .await
    .unwrap();
}

/// NOOP répond 250 et une commande inconnue répond 500.
#[tokio::test]
async fn noop_succeeds_and_unknown_command_is_refused() {
    timeout(LIMIT, async {
        let (server, _) = started_server().await;
        let mut client = authenticated_client(&server).await;
        assert!(client.command("NOOP").await.starts_with("250"));
        assert!(client.command("VRFY bob").await.starts_with("500"));
    })
    .await
    .unwrap();
}

/// QUIT répond 221 puis le serveur ferme la connexion.
#[tokio::test]
async fn quit_closes_the_connection() {
    timeout(LIMIT, async {
        let (server, _) = started_server().await;
        let mut client = authenticated_client(&server).await;
        assert!(client.command("QUIT").await.starts_with("221"));
        assert!(client.is_closed_by_peer().await);
    })
    .await
    .unwrap();
}

/// Une adresse sans chevrons reçoit 501 et rien n'est enregistré.
#[tokio::test]
async fn malformed_addresses_get_501() {
    timeout(LIMIT, async {
        let (server, _) = started_server().await;
        let mut client = authenticated_client(&server).await;
        assert!(
            client
                .command("MAIL FROM:alice@example.test")
                .await
                .starts_with("501")
        );
        client.command("MAIL FROM:<alice@example.test>").await;
        assert!(
            client
                .command("RCPT TO:bob@example.test")
                .await
                .starts_with("501")
        );
        assert!(server.received().is_empty());
    })
    .await
    .unwrap();
}

/// Deux messages successifs sur la même connexion sont reçus dans l'ordre.
#[tokio::test]
async fn two_messages_on_one_connection_are_received_in_order() {
    timeout(LIMIT, async {
        let (server, _) = started_server().await;
        let mut client = authenticated_client(&server).await;
        for body in ["One\r\n", "Two\r\n"] {
            let reply = smtp_support::flow::send_message(
                &mut client,
                "alice@example.test",
                &["bob@example.test"],
                body,
            )
            .await;
            assert!(reply.starts_with("250"), "{reply}");
        }
        let bodies: Vec<_> = server
            .received()
            .into_iter()
            .map(|mail| mail.data)
            .collect();
        assert_eq!(bodies, [b"One\r\n".to_vec(), b"Two\r\n".to_vec()]);
    })
    .await
    .unwrap();
}
