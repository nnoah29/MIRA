#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Règles et libellés locaux par la façade : enregistrement, statut après suppression, catalogue, aperçu.

mod support;
mod support_api;
mod support_imap;
mod support_label_api;
mod support_mirror;
mod support_rule_api;
mod support_write;

use mira_core::api::{LocalLabelRequest, Response, RuleRequest, RuleResponse};
use mira_core::rules::validate::RuleStatus;
use mira_vocabulary::{ConditionTest, LocalLabelId, Rule, RuleAction, RuleScope};
use support_imap::within;
use support_label_api::calls::{call, create, set};
use support_label_api::threads::seed_inbox_thread;
use support_rule_api::calls::{create as create_rule, list, rule_call};
use support_rule_api::samples::subject_rule;
use support_write::fixture::Fixture;

/// Règle « tous les comptes » qui pose ce libellé sur les messages dont l'objet contient « Sujet »
fn labeling_rule(label: &LocalLabelId) -> Rule {
    let mut rule = subject_rule("Étiqueter", "Sujet", RuleScope::AllAccounts);
    rule.actions = vec![RuleAction::AddLocalLabel {
        label: label.clone(),
    }];
    rule
}

/// Un libellé connu est accepté avec « tous les comptes » ; un libellé inconnu est refusé
#[tokio::test]
async fn rule_with_known_label_is_created_and_unknown_refused() {
    within(async {
        let f = Fixture::start().await;
        let urgent = create(f.engine(), "Urgent", "k-1").await.unwrap();
        let view = create_rule(f.engine(), labeling_rule(&urgent.id), "r-1").await;
        assert_eq!(view.unwrap().status, RuleStatus::Valid);
        let ghost = LocalLabelId::from_raw("zz");
        let refused = create_rule(f.engine(), labeling_rule(&ghost), "r-2").await;
        assert_eq!(refused.unwrap_err(), "invalid_input.parameter");
    })
    .await;
}

/// Supprimer le libellé rend la règle invalide, avec sa raison
#[tokio::test]
async fn rule_turns_invalid_when_its_label_is_deleted() {
    within(async {
        let f = Fixture::start().await;
        let urgent = create(f.engine(), "Urgent", "k-1").await.unwrap();
        create_rule(f.engine(), labeling_rule(&urgent.id), "r-1")
            .await
            .unwrap();
        let delete = LocalLabelRequest::DeleteLocalLabel {
            label: urgent.id.clone(),
        };
        call(f.engine(), delete).await.unwrap();
        let shown = list(f.engine()).await;
        assert!(matches!(shown[0].status, RuleStatus::Invalid { .. }));
    })
    .await;
}

/// Le catalogue propose la condition et les deux actions, rangées avec les libellés locaux, sans boîte exigée
#[tokio::test]
async fn catalog_lists_the_label_variants() {
    within(async {
        let f = Fixture::start().await;
        let answer = rule_call(f.engine(), RuleRequest::ListRuleCatalog)
            .await
            .unwrap();
        let Response::Rule(RuleResponse::Catalog(catalog)) = answer else {
            panic!("réponse inattendue");
        };
        let find = |entries: &[mira_core::api::CatalogEntry], kind: &str| {
            entries.iter().find(|e| e.kind == kind).cloned().unwrap()
        };
        let has = find(&catalog.conditions, "has_local_label");
        let add = find(&catalog.actions, "add_local_label");
        let remove = find(&catalog.actions, "remove_local_label");
        for entry in [&has, &add, &remove] {
            assert_eq!(entry.family, "local_labels");
            assert_eq!(entry.fields, ["label"]);
            assert!(!entry.needs_mailbox && !entry.single_account);
        }
    })
    .await;
}

/// L'aperçu lit les libellés réels des conversations : seul le message étiqueté est touché
#[tokio::test]
async fn preview_uses_the_real_labels_of_conversations() {
    within(async {
        let f = Fixture::start().await;
        let urgent = create(f.engine(), "Urgent", "k-1").await.unwrap();
        let (tagged, _) = seed_inbox_thread(&f.db, &f.account, "un", 501);
        seed_inbox_thread(&f.db, &f.account, "deux", 502);
        set(f.engine(), &urgent, &[tagged], true, "k-2")
            .await
            .unwrap();
        let mut rule = labeling_rule(&urgent.id);
        let test = ConditionTest::HasLocalLabel {
            label: urgent.id.clone(),
        };
        rule.alternatives[0].conditions[0].test = test;
        let request = RuleRequest::PreviewRule { rule, limit: 10 };
        let answer = rule_call(f.engine(), request).await.unwrap();
        let Response::Rule(RuleResponse::Preview(preview)) = answer else {
            panic!("réponse inattendue");
        };
        assert_eq!(preview.matched, 1);
    })
    .await;
}
