#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Domaine « rule » par la façade : refus structurés avant écriture, validation sans écriture, absence de secret.

mod support;
mod support_api;
mod support_imap;
mod support_mirror;
mod support_rule_api;
mod support_write;

use mira_core::api::{Response, RuleRequest, RuleResponse};
use mira_core::rules::validate::RuleStatus;
use mira_vocabulary::{AccountId, ErrorCategory, RuleScope};
use support_imap::within;
use support_rule_api::calls::{create, key, list, rule_call};
use support_rule_api::invalid::{invalid_rules, regex_rule};
use support_rule_api::samples::{changed_rules, subject_rule};
use support_rule_api::writes::update;
use support_write::fixture::Fixture;

/// Chaque règle incorrecte est refusée avec une erreur structurée ; rien n'est écrit ni annoncé
#[tokio::test]
async fn invalid_rules_are_refused_and_nothing_is_written() {
    within(async {
        let f = Fixture::start().await;
        for (rank, rule) in invalid_rules(&f.account).into_iter().enumerate() {
            let request = RuleRequest::CreateRule {
                rule,
                idempotency_key: key(&format!("k-{rank}")),
            };
            let error = rule_call(f.engine(), request).await.unwrap_err();
            assert_eq!(error.category, ErrorCategory::InvalidInput, "cas {rank}");
            assert_eq!(error.code, "invalid_input.parameter", "cas {rank}");
            assert!(!error.message.is_empty());
        }
        assert!(list(f.engine()).await.is_empty());
        assert!(changed_rules(f.engine()).await.is_empty());
    })
    .await;
}

/// Une mise à jour incorrecte, ou qui vise un compte inexistant, est refusée et la règle reste intacte
#[tokio::test]
async fn invalid_update_keeps_the_stored_rule() {
    within(async {
        let f = Fixture::start().await;
        let rule = subject_rule("Ok", "x", RuleScope::AllAccounts);
        let original = create(f.engine(), rule, "k-ok").await.unwrap().rule;
        let mut broken = original.clone();
        broken.actions.clear();
        let refused = update(f.engine(), broken, "k-up").await.unwrap_err();
        assert_eq!(refused, "invalid_input.parameter");
        let mut elsewhere = original.clone();
        elsewhere.scope = RuleScope::Accounts(vec![AccountId::from_raw("ghost")]);
        let missing = update(f.engine(), elsewhere, "k-up-2").await.unwrap_err();
        assert_eq!(missing, "not_found.account");
        assert_eq!(list(f.engine()).await[0].rule, original);
        assert_eq!(changed_rules(f.engine()).await.len(), 1);
    })
    .await;
}

/// Un compte de la portée inexistant donne `not_found.account`, pas une erreur de stockage
#[tokio::test]
async fn unknown_scope_account_is_not_found() {
    within(async {
        let f = Fixture::start().await;
        let ghost = RuleScope::Accounts(vec![AccountId::from_raw("ghost")]);
        let refused = create(f.engine(), subject_rule("Fantôme", "x", ghost), "k-1").await;
        assert_eq!(refused.unwrap_err(), "not_found.account");
        assert!(list(f.engine()).await.is_empty());
    })
    .await;
}

/// `ValidateRule` rend le statut, valide ou invalide avec la raison, sans rien enregistrer ni annoncer
#[tokio::test]
async fn validate_rule_answers_without_writing() {
    within(async {
        let f = Fixture::start().await;
        let good = subject_rule("Ok", "x", RuleScope::AllAccounts);
        let mut bad = good.clone();
        bad.actions.clear();
        let mut statuses = Vec::new();
        for rule in [good, bad] {
            let answer = rule_call(f.engine(), RuleRequest::ValidateRule { rule }).await;
            match answer.unwrap() {
                Response::Rule(RuleResponse::Status(status)) => statuses.push(status),
                other => panic!("réponse inattendue : {other:?}"),
            }
        }
        assert_eq!(statuses[0], RuleStatus::Valid);
        let RuleStatus::Invalid { reason } = &statuses[1] else {
            panic!("statut inattendu : {:?}", statuses[1]);
        };
        assert!(!reason.is_empty());
        assert!(list(f.engine()).await.is_empty());
        assert!(changed_rules(f.engine()).await.is_empty());
    })
    .await;
}

/// Ni les erreurs ni les événements ne répètent le contenu d'une règle ou un secret du compte
#[tokio::test]
async fn errors_and_events_carry_no_secret_nor_rule_content() {
    within(async {
        let f = Fixture::start().await;
        let request = RuleRequest::CreateRule {
            rule: regex_rule("Nom-SECRET-privé", "SECRET-("),
            idempotency_key: key("k-SECRET"),
        };
        let error = rule_call(f.engine(), request).await.unwrap_err();
        let text = serde_json::to_string(&error).unwrap();
        assert!(!text.contains("SECRET"), "{text}");
        let good = subject_rule("Nom-SECRET-privé", "SECRET", RuleScope::AllAccounts);
        create(f.engine(), good, "k-SECRET-2").await.unwrap();
        let events = support_api::queries::event_kinds(f.engine()).await;
        let text = serde_json::to_string(&events).unwrap();
        assert!(!text.contains("SECRET"), "{text}");
        assert!(!text.contains(support_write::PASSWORD));
    })
    .await;
}

/// Une portée qui répète un compte est dédoublonnée : la réponse est ce qui est enregistré
#[tokio::test]
async fn repeated_scope_account_is_deduplicated() {
    within(async {
        let f = Fixture::start().await;
        let twice = RuleScope::Accounts(vec![f.account.clone(), f.account.clone()]);
        let created = create(
            f.engine(),
            subject_rule("Double", "x", twice.clone()),
            "k-1",
        );
        let created = created.await.unwrap().rule;
        assert_eq!(created.scope, RuleScope::Accounts(vec![f.account.clone()]));
        assert_eq!(list(f.engine()).await[0].rule, created);
        let mut again = created.clone();
        again.scope = twice;
        let updated = update(f.engine(), again, "k-2").await.unwrap().rule;
        assert_eq!(updated, created);
        assert_eq!(list(f.engine()).await[0].rule, updated);
    })
    .await;
}
