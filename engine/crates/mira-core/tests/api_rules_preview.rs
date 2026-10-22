#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Domaine « rule » par la façade : aperçu sur les messages existants et catalogue des conditions et actions.

mod support;
mod support_api;
mod support_imap;
mod support_mirror;
mod support_rule_api;
mod support_write;

use mira_core::api::{CatalogEntry, Response, RuleCatalog, RuleRequest, RuleResponse};
use mira_vocabulary::{ConditionTest, MessageId, RuleAction, RuleScope};
use support_imap::within;
use support_rule_api::calls::{list, rule_call};
use support_rule_api::samples::{changed_rules, subject_rule, variant_names};
use support_rule_api::seed::seed_lots;
use support_write::fixture::Fixture;

/// L'aperçu compte les messages touchés hors Corbeille, plafonne les exemples à 50 et ne modifie rien
#[tokio::test]
async fn preview_counts_existing_messages_without_changing_anything() {
    within(async {
        let f = Fixture::start().await;
        seed_lots(&f.db, &f.account, 60);
        let mut rule = subject_rule("Lots", "lot", RuleScope::AllAccounts);
        rule.enabled = false;
        let request = RuleRequest::PreviewRule { rule, limit: 500 };
        let answer = rule_call(f.engine(), request).await.unwrap();
        let Response::Rule(RuleResponse::Preview(preview)) = answer else {
            panic!("réponse inattendue : {answer:?}");
        };
        assert_eq!(preview.matched, 60);
        assert_eq!(preview.scanned, 63);
        assert_eq!(preview.samples.len(), 50);
        assert_eq!(preview.samples[0], MessageId::from_raw("lot-59"));
        let lot =
            f.db.read(|tx| tx.get_message(&MessageId::from_raw("lot-1")));
        assert!(!lot.unwrap().unwrap().flags.seen);
        assert!(list(f.engine()).await.is_empty());
        assert!(changed_rules(f.engine()).await.is_empty());
    })
    .await;
}

/// La limite demandée plafonne les exemples, pas le nombre de messages touchés
#[tokio::test]
async fn preview_limit_caps_samples_only() {
    within(async {
        let f = Fixture::start().await;
        seed_lots(&f.db, &f.account, 5);
        let rule = subject_rule("Lots", "lot", RuleScope::AllAccounts);
        let request = RuleRequest::PreviewRule { rule, limit: 2 };
        let answer = rule_call(f.engine(), request).await.unwrap();
        let Response::Rule(RuleResponse::Preview(preview)) = answer else {
            panic!("réponse inattendue : {answer:?}");
        };
        assert_eq!(preview.matched, 5);
        let ids: Vec<&str> = preview.samples.iter().map(MessageId::as_str).collect();
        assert_eq!(ids, ["lot-4", "lot-3"]);
    })
    .await;
}

/// Un compte cité deux fois dans la portée ne compte pas deux fois ses messages
#[tokio::test]
async fn preview_counts_each_message_once_with_a_repeated_account() {
    within(async {
        let f = Fixture::start().await;
        seed_lots(&f.db, &f.account, 3);
        let twice = RuleScope::Accounts(vec![f.account.clone(), f.account.clone()]);
        let rule = subject_rule("Lots", "lot", twice);
        let request = RuleRequest::PreviewRule { rule, limit: 10 };
        let answer = rule_call(f.engine(), request).await.unwrap();
        let Response::Rule(RuleResponse::Preview(preview)) = answer else {
            panic!("réponse inattendue : {answer:?}");
        };
        assert_eq!((preview.matched, preview.scanned), (3, 6));
        assert_eq!(preview.samples.len(), 3);
    })
    .await;
}

/// Étiquettes triées d'une liste d'entrées du catalogue
fn kinds(entries: &[CatalogEntry]) -> Vec<String> {
    let mut kinds: Vec<String> = entries.iter().map(|entry| entry.kind.clone()).collect();
    kinds.sort();
    kinds
}

/// Le catalogue cite chaque variante de condition et d'action du moteur, avec les besoins de boîte
#[tokio::test]
async fn catalog_lists_every_condition_and_action() {
    within(async {
        let f = Fixture::start().await;
        let answer = rule_call(f.engine(), RuleRequest::ListRuleCatalog).await;
        let Response::Rule(RuleResponse::Catalog(catalog)) = answer.unwrap() else {
            panic!("réponse inattendue");
        };
        assert_eq!(kinds(&catalog.conditions), variant_names::<ConditionTest>());
        assert_eq!(kinds(&catalog.actions), variant_names::<RuleAction>());
        let entry = |kind: &str| find_entry(&catalog, kind);
        assert!(entry("move_to").needs_mailbox && entry("in_mailbox").needs_mailbox);
        assert!(!entry("trash").needs_mailbox && !entry("subject").needs_mailbox);
        assert!(entry("archive").single_account && !entry("archive").needs_mailbox);
        assert_eq!(entry("sender").fields, ["field", "matcher"]);
        assert_eq!(entry("sender").family, "sender");
    })
    .await;
}

/// Entrée du catalogue portant cette étiquette
fn find_entry(catalog: &RuleCatalog, kind: &str) -> CatalogEntry {
    let all = catalog.conditions.iter().chain(catalog.actions.iter());
    all.into_iter()
        .find(|entry| entry.kind == kind)
        .unwrap()
        .clone()
}
