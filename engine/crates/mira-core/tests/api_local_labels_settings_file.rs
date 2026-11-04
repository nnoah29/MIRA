#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Export et import YAML des libellés locaux avec leurs réglages : aller-retour, remplacement, événements.

mod support;
mod support_api;
mod support_imap;
mod support_label_api;
mod support_mirror;
mod support_write;

use mira_vocabulary::{CounterMode, LabelImportOutcome as Outcome, LabelSettings};
use support_imap::within;
use support_label_api::files::{
    create_colored, export_yaml, exported_items, file_of, import_file, outcomes,
};
use support_label_api::seeds::delete_all;
use support_label_api::settings::changed_count;
use support_label_api::tuning::{chosen, settings_of, tune};
use support_write::fixture::Fixture;

/// L'export porte les réglages de chaque libellé, valeurs par défaut comprises
#[tokio::test]
async fn export_carries_settings() {
    within(async {
        let f = Fixture::start().await;
        let made = create_colored(f.engine(), "Compta", 0xFF1E_90FF, "k-1").await;
        create_colored(f.engine(), "Voyages", 0xFF2E_8B57, "k-2").await;
        tune(f.engine(), made.id.as_str(), &chosen()).await;
        let items = exported_items(f.engine()).await;
        assert_eq!(items[0]["hidden"], true);
        assert_eq!(items[0]["counter"], "total");
        assert_eq!(items[1]["hidden"], false);
        assert_eq!(items[1]["notifications"], true);
    })
    .await;
}

/// Exporter, supprimer, importer : les réglages reviennent avec le libellé
#[tokio::test]
async fn round_trip_restores_settings() {
    within(async {
        let f = Fixture::start().await;
        let made = create_colored(f.engine(), "Compta", 0xFF1E_90FF, "k-1").await;
        tune(f.engine(), made.id.as_str(), &chosen()).await;
        let yaml = export_yaml(f.engine()).await.unwrap();
        delete_all(f.engine()).await;
        let report = import_file(f.engine(), &yaml, false, "k-imp")
            .await
            .unwrap();
        assert_eq!(outcomes(&report), vec![Outcome::Imported]);
        assert_eq!(settings_of(f.engine(), "Compta").await, chosen());
    })
    .await;
}

/// Un fichier sans réglage importe le libellé avec les réglages par défaut
#[tokio::test]
async fn old_file_without_settings_gives_defaults() {
    within(async {
        let f = Fixture::start().await;
        let file = file_of(vec![
            serde_json::json!({"name": "Ancien", "color": "#ff102030"}),
        ]);
        import_file(f.engine(), &file, false, "k-imp")
            .await
            .unwrap();
        assert_eq!(
            settings_of(f.engine(), "Ancien").await,
            LabelSettings::default()
        );
    })
    .await;
}

/// Sans remplacement, un libellé présent garde ses réglages ; avec remplacement, il prend ceux du fichier
#[tokio::test]
async fn replace_applies_settings_only_when_asked() {
    within(async {
        let f = Fixture::start().await;
        create_colored(f.engine(), "Compta", 0xFF1E_90FF, "k-1").await;
        let file = file_of(vec![serde_json::json!({
            "name": "Compta", "color": "#ff1e90ff", "hidden": true, "counter": "none"
        })]);
        let kept = import_file(f.engine(), &file, false, "k-a").await.unwrap();
        assert_eq!(outcomes(&kept), vec![Outcome::AlreadyPresent]);
        assert_eq!(
            settings_of(f.engine(), "Compta").await,
            LabelSettings::default()
        );
        let done = import_file(f.engine(), &file, true, "k-b").await.unwrap();
        assert_eq!(outcomes(&done), vec![Outcome::Replaced]);
        let after = settings_of(f.engine(), "Compta").await;
        assert!(after.hidden && after.counter == CounterMode::None && after.notifications);
    })
    .await;
}

/// Un remplacement qui ne change rien est « déjà présent » et n'annonce rien
#[tokio::test]
async fn identical_replace_writes_and_announces_nothing() {
    within(async {
        let f = Fixture::start().await;
        let made = create_colored(f.engine(), "Compta", 0xFF1E_90FF, "k-1").await;
        tune(f.engine(), made.id.as_str(), &chosen()).await;
        let yaml = export_yaml(f.engine()).await.unwrap();
        let before = changed_count(f.engine()).await;
        let report = import_file(f.engine(), &yaml, true, "k-imp").await.unwrap();
        assert_eq!(outcomes(&report), vec![Outcome::AlreadyPresent]);
        assert_eq!(changed_count(f.engine()).await, before);
    })
    .await;
}

/// Un réglage écrit par l'import est annoncé par l'événement des réglages
#[tokio::test]
async fn import_of_settings_announces_settings_change() {
    within(async {
        let f = Fixture::start().await;
        let file = file_of(vec![serde_json::json!({"name": "Compta", "hidden": true})]);
        let before = changed_count(f.engine()).await;
        import_file(f.engine(), &file, false, "k-imp")
            .await
            .unwrap();
        assert!(changed_count(f.engine()).await > before);
    })
    .await;
}
