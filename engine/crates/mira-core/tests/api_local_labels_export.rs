#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Export YAML des libellés locaux par la façade : contenu, enveloppe, liste vide, aucun événement.

mod support;
mod support_api;
mod support_imap;
mod support_label_api;
mod support_mirror;
mod support_write;

use support_imap::within;
use support_label_api::calls::{changed_count, labels};
use support_label_api::files::{create_colored, export_yaml, exported_items};
use support_write::fixture::Fixture;

/// Chaque libellé sort avec son nom et sa couleur `#aarrggbb`, dans l'ordre de la liste
#[tokio::test]
async fn export_lists_labels_with_name_and_hex_color() {
    within(async {
        let f = Fixture::start().await;
        create_colored(f.engine(), "Voyages", 0xFF2E_8B57, "k-1").await;
        create_colored(f.engine(), "Compta", 0xFF1E_90FF, "k-2").await;
        create_colored(f.engine(), "Banque", 0x80FF_AA00, "k-3").await;
        let items = exported_items(f.engine()).await;
        let seen: Vec<_> = items
            .iter()
            .map(|i| (i["name"].as_str().unwrap(), i["color"].as_str().unwrap()))
            .collect();
        assert_eq!(
            seen,
            [
                ("Banque", "#80ffaa00"),
                ("Compta", "#ff1e90ff"),
                ("Voyages", "#ff2e8b57")
            ]
        );
        assert_eq!(labels(f.engine()).await.len(), 3);
    })
    .await;
}

/// Le fichier porte l'en-tête du type « local_labels » en version 1
#[tokio::test]
async fn export_writes_envelope_kind_local_labels_version_1() {
    within(async {
        let f = Fixture::start().await;
        let yaml = export_yaml(f.engine()).await.unwrap();
        assert!(yaml.contains("kind: local_labels"), "{yaml}");
        assert!(yaml.contains("version: 1"), "{yaml}");
    })
    .await;
}

/// Sans libellé, l'export est un fichier valide à liste vide
#[tokio::test]
async fn export_of_no_label_is_valid_empty_list() {
    within(async {
        let f = Fixture::start().await;
        assert!(exported_items(f.engine()).await.is_empty());
    })
    .await;
}

/// Exporter n'émet aucun événement
#[tokio::test]
async fn export_emits_no_event() {
    within(async {
        let f = Fixture::start().await;
        create_colored(f.engine(), "Compta", 0xFF1E_90FF, "k-1").await;
        let before = changed_count(f.engine()).await;
        export_yaml(f.engine()).await.unwrap();
        assert_eq!(changed_count(f.engine()).await, before);
    })
    .await;
}

/// Le fichier ne contient ni identifiant de libellé ni conversation
#[tokio::test]
async fn export_contains_no_conversation_or_id() {
    within(async {
        let f = Fixture::start().await;
        let label = create_colored(f.engine(), "Compta", 0xFF1E_90FF, "k-1").await;
        let yaml = export_yaml(f.engine()).await.unwrap();
        assert!(!yaml.contains(label.id.as_str()), "{yaml}");
        assert!(!yaml.contains("thread"), "{yaml}");
    })
    .await;
}
