#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Libellés propres au service et recherche de texte.

mod support_imap;

use mira_testkit::imap::CapabilitySet;
use support_imap::env::start;
use support_imap::logins::selected;
use support_imap::mail::{MAIL, mail};
use support_imap::within;

/// La recherche de texte rend les UID des messages qui contiennent la requête
#[tokio::test]
async fn search_text_finds_matching_messages() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store
            .deliver("alice", "INBOX", &mail("facture mars", "a@x"), &[]);
        env.store
            .deliver("alice", "INBOX", &mail("vacances", "b@x"), &[]);
        env.store
            .deliver("alice", "INBOX", &mail("facture avril", "c@x"), &[]);
        let mut client = selected(&env).await;
        assert_eq!(client.search_text("facture").await.unwrap(), vec![1, 3]);
        assert!(
            client
                .search_text("introuvable \"x\"")
                .await
                .unwrap()
                .is_empty()
        );
        assert_eq!(
            client.search_text("a\r\nb").await.unwrap_err().code,
            "invalid_input.parameter"
        );
    })
    .await;
}

/// Ajouter un libellé met le message dans la boîte correspondante, le retirer l'en sort
#[tokio::test]
async fn add_and_remove_labels() {
    within(async {
        let env = start(CapabilitySet::full().with_label_extensions()).await;
        env.store
            .deliver_with_labels("alice", &["INBOX"], MAIL, 10, 20);
        let mut client = selected(&env).await;
        let labels = vec!["Work".to_string()];
        client.add_labels(&[1], &labels).await.unwrap();
        assert_eq!(env.store.messages("alice", "Work").len(), 1);
        client.remove_labels(&[1], &labels).await.unwrap();
        assert!(env.store.messages("alice", "Work").is_empty());
        assert_eq!(env.store.messages("alice", "INBOX").len(), 1);
    })
    .await;
}

/// Sans l'extension de libellés, l'opération est refusée localement
#[tokio::test]
async fn labels_require_the_extension() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        let mut client = selected(&env).await;
        let error = client
            .add_labels(&[1], &["Work".to_string()])
            .await
            .unwrap_err();
        assert_eq!(error.code, "server.extension_missing");
    })
    .await;
}
