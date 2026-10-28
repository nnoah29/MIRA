#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Règle témoin utilisant compte, boîte, libellé et modèle, et sa forme portable.

use mira_core::portability::rules::to_portable;
use mira_vocabulary::{
    AccountId, Condition, ConditionGroup, ConditionTest, LocalLabelId, MailboxId, Rule, RuleAction,
    RuleId, RuleScope, TemplateId, TextMatcher, TextOp,
};
use serde_json::Value;

use super::fake_refs::FakeRefs;

/// Conditions de la règle témoin : texte, boîte, compte et libellé
fn conditions() -> Vec<Condition> {
    let matcher = TextMatcher {
        op: TextOp::Contains,
        values: vec!["facture".to_string()],
    };
    vec![
        ConditionTest::Subject { matcher },
        ConditionTest::InMailbox {
            mailbox: MailboxId::from_raw("m1"),
        },
        ConditionTest::FromAccount {
            account: AccountId::from_raw("a1"),
        },
        ConditionTest::HasLocalLabel {
            label: LocalLabelId::from_raw("l1"),
        },
    ]
    .into_iter()
    .map(|test| Condition {
        negated: false,
        test,
    })
    .collect()
}

/// Règle témoin complète, portée sur le compte `a1`
pub fn full_rule() -> Rule {
    Rule {
        id: RuleId::from_raw("r1"),
        name: "Factures".to_string(),
        enabled: true,
        scope: RuleScope::Accounts(vec![AccountId::from_raw("a1")]),
        alternatives: vec![ConditionGroup {
            conditions: conditions(),
        }],
        actions: vec![
            RuleAction::MoveTo {
                mailbox: MailboxId::from_raw("m1"),
            },
            RuleAction::AddLocalLabel {
                label: LocalLabelId::from_raw("l1"),
            },
            RuleAction::ReplyWithTemplate {
                template: TemplateId::from_raw("t1"),
            },
        ],
        schema_version: 1,
    }
}

/// Forme portable de la règle témoin
pub fn portable() -> Value {
    to_portable(&full_rule(), &FakeRefs::default()).unwrap()
}

/// Remplace un champ de la forme portable
pub fn with_field(mut value: Value, field: &str, new: Value) -> Value {
    value[field] = new;
    value
}
