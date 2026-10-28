#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Import YAML des règles par la façade : restauration, doublons ignorés ou remplacés.

mod support;
mod support_api;
mod support_imap;
mod support_mirror;
mod support_rule_api;
mod support_write;

use mira_vocabulary::RuleImportOutcome as Outcome;
use support_imap::within;
use support_rule_api::calls::list;
use support_rule_api::portability::{
    export_yaml, exported_items, file_of, import, outcomes, variant,
};
use support_rule_api::writes::{create_named, delete, set_enabled, update};
use support_write::fixture::Fixture;

/// Exporter, vider, importer : mêmes règles, mêmes identifiants, même ordre, même état
#[tokio::test]
async fn round_trip_restores_identical_rules_and_order() {
    within(async {
        let f = Fixture::start().await;
        let ids = create_named(f.engine(), &["A", "B", "C"]).await;
        set_enabled(f.engine(), &ids[1], false, "k-off")
            .await
            .unwrap();
        let before = list(f.engine()).await;
        let yaml = export_yaml(f.engine()).await.unwrap();
        for (n, id) in ids.iter().enumerate() {
            delete(f.engine(), id, &format!("k-del-{n}")).await.unwrap();
        }
        let report = import(f.engine(), &yaml, false, "k-imp").await.unwrap();
        assert_eq!(outcomes(&report), vec![Outcome::Imported; 3]);
        assert_eq!(list(f.engine()).await, before);
    })
    .await;
}

/// Une règle déjà présente est signalée et laissée intacte, même modifiée depuis la sauvegarde
#[tokio::test]
async fn existing_rule_is_reported_already_present_and_untouched() {
    within(async {
        let f = Fixture::start().await;
        create_named(f.engine(), &["A"]).await;
        let yaml = export_yaml(f.engine()).await.unwrap();
        let mut changed = list(f.engine()).await[0].rule.clone();
        changed.name = "A modifiée".to_string();
        update(f.engine(), changed, "k-up").await.unwrap();
        let report = import(f.engine(), &yaml, false, "k-imp").await.unwrap();
        assert_eq!(outcomes(&report), vec![Outcome::AlreadyPresent]);
        assert_eq!(list(f.engine()).await[0].rule.name, "A modifiée");
    })
    .await;
}

/// Avec le remplacement, la règle du fichier écrase l'existante et garde sa position
#[tokio::test]
async fn replace_existing_overwrites_and_keeps_position() {
    within(async {
        let f = Fixture::start().await;
        create_named(f.engine(), &["A", "Z"]).await;
        let yaml = export_yaml(f.engine()).await.unwrap();
        let mut changed = list(f.engine()).await[0].rule.clone();
        changed.name = "A modifiée".to_string();
        update(f.engine(), changed, "k-up").await.unwrap();
        let report = import(f.engine(), &yaml, true, "k-imp").await.unwrap();
        assert_eq!(
            outcomes(&report),
            vec![Outcome::Replaced, Outcome::Replaced]
        );
        let names: Vec<_> = list(f.engine())
            .await
            .into_iter()
            .map(|v| v.rule.name)
            .collect();
        assert_eq!(names, ["A", "Z"]);
    })
    .await;
}

/// Un remplacement dont une référence est introuvable laisse la règle existante intacte
#[tokio::test]
async fn replace_with_unresolvable_reference_leaves_existing_rule_intact() {
    within(async {
        let f = Fixture::start().await;
        create_named(f.engine(), &["A"]).await;
        let mut item = exported_items(f.engine()).await.remove(0);
        item["name"] = "Remplaçante".into();
        item["actions"] = serde_json::json!([{"type": "add_local_label", "label": "Inconnu"}]);
        let report = import(f.engine(), &file_of(vec![item]), true, "k-imp")
            .await
            .unwrap();
        assert!(matches!(outcomes(&report)[0], Outcome::Refused { .. }));
        assert_eq!(list(f.engine()).await[0].rule.name, "A");
    })
    .await;
}

/// Une règle refusée n'empêche pas l'import des suivantes
#[tokio::test]
async fn refused_rule_does_not_block_following_rules() {
    within(async {
        let f = Fixture::start().await;
        create_named(f.engine(), &["A"]).await;
        let base = exported_items(f.engine()).await.remove(0);
        let mut bad = variant(&base, "bad-1", "Cassée");
        bad["actions"] = serde_json::json!([{"type": "add_local_label", "label": "Inconnu"}]);
        let good = variant(&base, "good-1", "Bonne");
        let report = import(f.engine(), &file_of(vec![bad, good]), false, "k-imp")
            .await
            .unwrap();
        assert!(matches!(outcomes(&report)[0], Outcome::Refused { .. }));
        assert_eq!(outcomes(&report)[1], Outcome::Imported);
        assert_eq!(list(f.engine()).await.len(), 2);
    })
    .await;
}
