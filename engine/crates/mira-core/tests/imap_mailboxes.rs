#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Liste des boîtes, sélection et marqueurs, coupure de connexion.

mod support_imap;

use mira_testkit::imap::CapabilitySet;
use support_imap::env::start;
use support_imap::logins::connected;
use support_imap::mail::MAIL;
use support_imap::within;

/// La liste donne les noms, le séparateur et les attributs (usage spécial, enfants)
#[tokio::test]
async fn list_mailboxes_with_attributes() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.create_mailbox("alice", "Sent", Some("\\Sent"));
        env.store.create_mailbox("alice", "Archive/2023", None);
        env.store.create_mailbox("alice", "Archive", None);
        let mut client = connected(&env).await;
        let boxes = client.list_mailboxes().await.unwrap();
        let find = |name: &str| boxes.iter().find(|b| b.name == name).unwrap();
        assert_eq!(find("INBOX").delimiter.as_deref(), Some("/"));
        assert!(find("Sent").attributes.contains(&"\\Sent".to_string()));
        assert!(
            find("Archive")
                .attributes
                .contains(&"\\HasChildren".to_string())
        );
        assert!(
            find("Archive/2023")
                .attributes
                .contains(&"\\HasNoChildren".to_string())
        );
    })
    .await;
}

/// SELECT rend les marqueurs de synchronisation et les drapeaux de la boîte
#[tokio::test]
async fn select_reports_sync_markers() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.deliver("alice", "INBOX", MAIL, &[]);
        env.store.deliver("alice", "INBOX", MAIL, &["\\Seen"]);
        let mut client = connected(&env).await;
        let selected = client.select("INBOX", None).await.unwrap();
        assert_eq!(selected.exists, 2);
        assert_eq!(selected.uid_next, 3);
        assert_eq!(
            selected.uid_validity,
            env.store.uidvalidity("alice", "INBOX").unwrap()
        );
        assert_eq!(
            selected.highest_modseq,
            env.store.highest_modseq("alice", "INBOX")
        );
        assert!(selected.flags.contains(&"\\Seen".to_string()));
        assert!(selected.vanished.is_empty() && selected.changed.is_empty());
    })
    .await;
}

/// Sans CONDSTORE, aucun numéro de modification n'est rendu
#[tokio::test]
async fn select_without_condstore_has_no_modseq() {
    within(async {
        let env = start(CapabilitySet::minimal()).await;
        let mut client = connected(&env).await;
        let selected = client.select("INBOX", None).await.unwrap();
        assert_eq!(selected.exists, 0);
        assert!(selected.highest_modseq.is_none());
    })
    .await;
}

/// Une boîte inexistante donne server.mailbox_missing
#[tokio::test]
async fn select_missing_mailbox_is_reported() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = connected(&env).await;
        let error = client.select("Inconnue", None).await.unwrap_err();
        assert_eq!(error.code, "server.mailbox_missing");
    })
    .await;
}

/// Les noms avec guillemets et barre oblique inverse font l'aller-retour
#[tokio::test]
async fn awkward_mailbox_names_round_trip() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store
            .create_mailbox("alice", "Dossier \"x\" \\ y", None);
        env.store.deliver("alice", "Dossier \"x\" \\ y", MAIL, &[]);
        let mut client = connected(&env).await;
        let boxes = client.list_mailboxes().await.unwrap();
        assert!(boxes.iter().any(|b| b.name == "Dossier \"x\" \\ y"));
        let selected = client.select("Dossier \"x\" \\ y", None).await.unwrap();
        assert_eq!(selected.exists, 1);
        let rejected = client.select("a\r\nb", None).await.unwrap_err();
        assert_eq!(rejected.code, "invalid_input.parameter");
    })
    .await;
}

/// Une coupure de connexion donne network.connection_lost, sans réponse bloquée
#[tokio::test]
async fn dropped_connection_is_reported() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = connected(&env).await;
        env.server.drop_all_connections();
        let error = client.list_mailboxes().await.unwrap_err();
        assert_eq!(error.code, "network.connection_lost");
        assert!(error.retryable);
    })
    .await;
}
