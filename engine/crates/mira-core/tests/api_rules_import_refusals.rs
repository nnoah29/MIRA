#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Import YAML des règles : raisons de refus, doublon interne au fichier, ordre des ajouts.

mod support;
mod support_api;
mod support_imap;
mod support_mirror;
mod support_rule_api;
mod support_write;

use mira_vocabulary::{RuleImportOutcome as Outcome, RuleImportRefusal as Refusal};
use support_imap::within;
use support_rule_api::calls::list;
use support_rule_api::portability::{exported_items, file_of, import, outcomes, variant};
use support_rule_api::refusals::{assert_refusal_report, refusal_items};
use support_rule_api::writes::create_named;
use support_write::fixture::Fixture;

/// Chaque cause de refus est rapportée avec la position et le nom de la règle
#[tokio::test]
async fn each_refusal_reason_is_reported_with_position_and_name() {
    within(async {
        let f = Fixture::start().await;
        create_named(f.engine(), &["A"]).await;
        let base = exported_items(f.engine()).await.remove(0);
        let yaml = file_of(refusal_items(&base));
        let report = import(f.engine(), &yaml, false, "k-imp").await.unwrap();
        assert_refusal_report(&report);
        assert_eq!(list(f.engine()).await.len(), 1);
    })
    .await;
}

/// Deux règles du fichier de même identifiant : la seconde est refusée, une seule est écrite
#[tokio::test]
async fn duplicate_id_inside_file_refuses_second() {
    within(async {
        let f = Fixture::start().await;
        create_named(f.engine(), &["A"]).await;
        let base = exported_items(f.engine()).await.remove(0);
        let items = vec![
            variant(&base, "same", "Première"),
            variant(&base, "same", "Seconde"),
        ];
        let report = import(f.engine(), &file_of(items), false, "k-imp")
            .await
            .unwrap();
        assert_eq!(outcomes(&report)[0], Outcome::Imported);
        assert!(matches!(
            outcomes(&report)[1],
            Outcome::Refused {
                reason: Refusal::InvalidRule { .. }
            }
        ));
        assert_eq!(list(f.engine()).await.len(), 2);
    })
    .await;
}

/// Les règles ajoutées suivent les existantes, dans l'ordre du fichier
#[tokio::test]
async fn new_rules_are_appended_after_existing_in_file_order() {
    within(async {
        let f = Fixture::start().await;
        create_named(f.engine(), &["A"]).await;
        let base = exported_items(f.engine()).await.remove(0);
        let items = vec![variant(&base, "n2", "Deux"), variant(&base, "n1", "Un")];
        import(f.engine(), &file_of(items), false, "k-imp")
            .await
            .unwrap();
        let names: Vec<_> = list(f.engine())
            .await
            .into_iter()
            .map(|v| v.rule.name)
            .collect();
        assert_eq!(names, ["A", "Deux", "Un"]);
    })
    .await;
}
