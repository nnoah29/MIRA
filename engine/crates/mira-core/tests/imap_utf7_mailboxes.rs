#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Noms de boîtes en UTF-7 modifié (RFC 3501 §5.1.3) : le moteur ne voit que du texte lisible.

mod support_imap;

use mira_testkit::imap::CapabilitySet;
use support_imap::env::start;
use support_imap::logins::connected;
use support_imap::mail::MAIL;
use support_imap::within;

const SENT_WIRE: &str = "&AMk-l&AOk-ments envoy&AOk-s";
const SENT_TEXT: &str = "Éléments envoyés";

/// Une boîte annoncée en UTF-7 est rendue en texte lisible
#[tokio::test]
async fn listed_mailbox_names_are_decoded() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.create_mailbox("alice", SENT_WIRE, None);
        let mut client = connected(&env).await;
        let names: Vec<String> = client
            .list_mailboxes()
            .await
            .unwrap()
            .into_iter()
            .map(|mailbox| mailbox.name)
            .collect();
        assert!(names.contains(&SENT_TEXT.to_string()), "{names:?}");
    })
    .await;
}

/// Ouvrir la boîte par son nom lisible envoie le nom UTF-7 que le serveur connaît
#[tokio::test]
async fn selecting_a_decoded_name_reaches_the_server_mailbox() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.create_mailbox("alice", SENT_WIRE, None);
        env.store.deliver("alice", SENT_WIRE, MAIL, &[]);
        let mut client = connected(&env).await;
        let selected = client.select(SENT_TEXT, None).await.unwrap();
        assert_eq!(selected.exists, 1);
    })
    .await;
}

/// Créer, renommer et supprimer écrivent des noms UTF-7, `&` littéral compris
#[tokio::test]
async fn administration_commands_send_encoded_names() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = connected(&env).await;
        client.create_mailbox("Reçus R&D").await.unwrap();
        client.rename_mailbox("Reçus R&D", "Reçus €").await.unwrap();
        client.delete_mailbox("Reçus €").await.unwrap();
        let log = env.store.command_log().join("\n");
        assert!(log.contains("CREATE \"Re&AOc-us R&-D\""), "{log}");
        assert!(
            log.contains("RENAME \"Re&AOc-us R&-D\" \"Re&AOc-us &IKw-\""),
            "{log}"
        );
        assert!(log.contains("DELETE \"Re&AOc-us &IKw-\""), "{log}");
    })
    .await;
}

/// Un nom hors plan de base (emoji) fait l'aller-retour sans perte
#[tokio::test]
async fn names_outside_the_basic_plane_round_trip() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        let mut client = connected(&env).await;
        client.create_mailbox("Fête 😀").await.unwrap();
        let names: Vec<String> = client
            .list_mailboxes()
            .await
            .unwrap()
            .into_iter()
            .map(|mailbox| mailbox.name)
            .collect();
        assert!(names.contains(&"Fête 😀".to_string()), "{names:?}");
    })
    .await;
}

/// Un nom mal formé venant du serveur est gardé tel quel, sans erreur
#[tokio::test]
async fn malformed_server_names_are_kept_verbatim() {
    within(async {
        let env = start(CapabilitySet::full()).await;
        env.store.create_mailbox("alice", "Cassé &AMk", None);
        let mut client = connected(&env).await;
        let names: Vec<String> = client
            .list_mailboxes()
            .await
            .unwrap()
            .into_iter()
            .map(|mailbox| mailbox.name)
            .collect();
        assert!(names.contains(&"Cassé &AMk".to_string()), "{names:?}");
    })
    .await;
}
