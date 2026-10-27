#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Import YAML des libellés locaux : restauration, doublons ignorés ou remplacés.

mod support;
mod support_api;
mod support_imap;
mod support_label_api;
mod support_mirror;
mod support_write;

use mira_core::api::LocalLabelRequest;
use mira_vocabulary::LabelImportOutcome as Outcome;
use serde_json::json;
use support_imap::within;
use support_label_api::calls::{call, changed_count, labels};
use support_label_api::files::{create_colored, export_yaml, file_of, import_file, outcomes};
use support_label_api::seeds::{delete_all, pairs, seed_three};
use support_write::fixture::Fixture;

/// Exporter, supprimer, importer : mêmes noms et mêmes couleurs (les identifiants sont neufs)
#[tokio::test]
async fn round_trip_restores_names_and_colors() {
    within(async {
        let f = Fixture::start().await;
        seed_three(f.engine()).await;
        let before = pairs(f.engine()).await;
        let yaml = export_yaml(f.engine()).await.unwrap();
        delete_all(f.engine()).await;
        let report = import_file(f.engine(), &yaml, false, "k-imp")
            .await
            .unwrap();
        assert_eq!(outcomes(&report), vec![Outcome::Imported; 3]);
        assert_eq!(pairs(f.engine()).await, before);
    })
    .await;
}

/// Un libellé déjà présent est signalé et laissé tel quel, même recoloré depuis la sauvegarde
#[tokio::test]
async fn existing_label_is_reported_already_present_and_untouched() {
    within(async {
        let f = Fixture::start().await;
        let label = create_colored(f.engine(), "Compta", 0xFF11_1111, "k-1").await;
        let yaml = export_yaml(f.engine()).await.unwrap();
        let recolor = LocalLabelRequest::RecolorLocalLabel {
            label: label.id,
            color: 0xFF22_2222,
        };
        call(f.engine(), recolor).await.unwrap();
        let report = import_file(f.engine(), &yaml, false, "k-imp")
            .await
            .unwrap();
        assert_eq!(outcomes(&report), vec![Outcome::AlreadyPresent]);
        assert_eq!(labels(f.engine()).await[0].color, 0xFF22_2222);
    })
    .await;
}

/// Avec le remplacement, le libellé existant prend le nom (casse comprise) et la couleur du fichier
#[tokio::test]
async fn replace_existing_updates_name_case_and_color() {
    within(async {
        let f = Fixture::start().await;
        let before = create_colored(f.engine(), "compta", 0xFF11_1111, "k-1").await;
        let yaml = file_of(vec![json!({"name": "Compta", "color": "#ff222222"})]);
        let report = import_file(f.engine(), &yaml, true, "k-imp").await.unwrap();
        assert_eq!(outcomes(&report), vec![Outcome::Replaced]);
        let after = labels(f.engine()).await;
        assert_eq!(
            (after[0].id.clone(), after[0].name.as_str(), after[0].color),
            (before.id, "Compta", 0xFF22_2222)
        );
    })
    .await;
}

/// Réimporter sa propre sauvegarde avec le remplacement ne change rien : déjà présent, aucun événement
#[tokio::test]
async fn replace_with_identical_label_is_already_present_without_event() {
    within(async {
        let f = Fixture::start().await;
        create_colored(f.engine(), "Compta", 0xFF1E_90FF, "k-1").await;
        create_colored(f.engine(), "Voyages", 0xFF2E_8B57, "k-2").await;
        let yaml = export_yaml(f.engine()).await.unwrap();
        let before = changed_count(f.engine()).await;
        let report = import_file(f.engine(), &yaml, true, "k-imp").await.unwrap();
        assert_eq!(outcomes(&report), vec![Outcome::AlreadyPresent; 2]);
        assert_eq!(changed_count(f.engine()).await, before);
    })
    .await;
}

/// Les noms se rapprochent sans tenir compte de la casse ni des espaces de bord
#[tokio::test]
async fn labels_are_matched_ignoring_case_and_edge_spaces() {
    within(async {
        let f = Fixture::start().await;
        create_colored(f.engine(), "Compta", 0xFF1E_90FF, "k-1").await;
        let yaml = file_of(vec![json!({"name": "  COMPTA ", "color": "#ff1e90ff"})]);
        let report = import_file(f.engine(), &yaml, false, "k-imp")
            .await
            .unwrap();
        assert_eq!(outcomes(&report), vec![Outcome::AlreadyPresent]);
        assert_eq!(labels(f.engine()).await.len(), 1);
    })
    .await;
}
