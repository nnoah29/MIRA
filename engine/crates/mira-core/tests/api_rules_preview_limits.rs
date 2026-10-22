#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Plafonds de forme par la façade : refusés à l'enregistrement comme à l'aperçu ; l'aperçu tolère un brouillon sans action.

mod support;
mod support_api;
mod support_imap;
mod support_mirror;
mod support_rule_api;
mod support_write;

use mira_core::api::{Response, RuleRequest, RuleResponse};
use mira_vocabulary::{ErrorCategory, RuleScope};
use support_imap::within;
use support_rule_api::calls::{create, list, rule_call};
use support_rule_api::invalid::oversized_rules;
use support_rule_api::samples::subject_rule;
use support_rule_api::seed::seed_lots;
use support_write::fixture::Fixture;

/// Valeur ou regex trop longue, trop de valeurs, nom trop long : l'aperçu est refusé avec `invalid_input.parameter`
#[tokio::test]
async fn oversized_preview_is_refused() {
    within(async {
        let f = Fixture::start().await;
        seed_lots(&f.db, &f.account, 3);
        for (rank, rule) in oversized_rules().into_iter().enumerate() {
            let request = RuleRequest::PreviewRule { rule, limit: 10 };
            let error = rule_call(f.engine(), request).await.unwrap_err();
            assert_eq!(error.category, ErrorCategory::InvalidInput, "cas {rank}");
            assert_eq!(error.code, "invalid_input.parameter", "cas {rank}");
        }
    })
    .await;
}

/// Les mêmes règles sont refusées à l'enregistrement, sans rien écrire
#[tokio::test]
async fn oversized_rule_is_refused_at_save() {
    within(async {
        let f = Fixture::start().await;
        for (rank, rule) in oversized_rules().into_iter().enumerate() {
            let refused = create(f.engine(), rule, &format!("k-{rank}")).await;
            assert_eq!(
                refused.unwrap_err(),
                "invalid_input.parameter",
                "cas {rank}"
            );
        }
        assert!(list(f.engine()).await.is_empty());
    })
    .await;
}

/// Un brouillon sans action reste prévisualisable : seuls les contrôles de forme s'appliquent
#[tokio::test]
async fn preview_accepts_a_draft_without_actions() {
    within(async {
        let f = Fixture::start().await;
        seed_lots(&f.db, &f.account, 3);
        let mut draft = subject_rule("Lots", "lot", RuleScope::AllAccounts);
        draft.actions.clear();
        let request = RuleRequest::PreviewRule {
            rule: draft,
            limit: 10,
        };
        let answer = rule_call(f.engine(), request).await.unwrap();
        let Response::Rule(RuleResponse::Preview(preview)) = answer else {
            panic!("réponse inattendue : {answer:?}");
        };
        assert_eq!(preview.matched, 3);
    })
    .await;
}
