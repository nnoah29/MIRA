#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Export YAML des règles par la façade : ordre, état, enveloppe, liste vide, aucun événement.

mod support;
mod support_api;
mod support_imap;
mod support_mirror;
mod support_rule_api;
mod support_write;

use support_api::queries::event_kinds;
use support_imap::within;
use support_rule_api::calls::create;
use support_rule_api::portability::{export_yaml, exported_items};
use support_rule_api::samples::subject_rule;
use support_rule_api::writes::{create_named, set_enabled};
use support_write::fixture::Fixture;

/// Les règles sortent dans l'ordre d'évaluation, avec leur état activé ou désactivé
#[tokio::test]
async fn export_lists_rules_in_evaluation_order_with_enabled_state() {
    within(async {
        let f = Fixture::start().await;
        let ids = create_named(f.engine(), &["Première", "Seconde"]).await;
        set_enabled(f.engine(), &ids[1], false, "k-off")
            .await
            .unwrap();
        let items = exported_items(f.engine()).await;
        let names: Vec<_> = items
            .iter()
            .map(|item| item["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["Première", "Seconde"]);
        assert_eq!(items[0]["enabled"], true);
        assert_eq!(items[1]["enabled"], false);
    })
    .await;
}

/// Le fichier porte l'en-tête du type « rules » en version 1
#[tokio::test]
async fn export_writes_envelope_kind_rules_version_1() {
    within(async {
        let f = Fixture::start().await;
        let yaml = export_yaml(f.engine()).await.unwrap();
        assert!(yaml.contains("mira_export:"), "{yaml}");
        assert!(yaml.contains("kind: rules"), "{yaml}");
        assert!(yaml.contains("version: 1"), "{yaml}");
    })
    .await;
}

/// Sans règle, l'export est un fichier valide à liste vide
#[tokio::test]
async fn export_of_no_rule_is_valid_empty_list() {
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
        let rule = subject_rule(
            "Factures",
            "Facture",
            mira_vocabulary::RuleScope::AllAccounts,
        );
        create(f.engine(), rule, "k-1").await.unwrap();
        let before = event_kinds(f.engine()).await.len();
        export_yaml(f.engine()).await.unwrap();
        assert_eq!(event_kinds(f.engine()).await.len(), before);
    })
    .await;
}
