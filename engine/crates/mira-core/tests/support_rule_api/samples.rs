//! Règles d'exemple, événements de règle et noms de variantes tirés de serde.

use mira_core::api::Engine;
use mira_vocabulary::{
    AccountId, Condition, ConditionGroup, ConditionTest, EventKind, MailboxId, Rule, RuleAction,
    RuleId, RuleScope, TextMatcher, TextOp,
};
use serde::de::DeserializeOwned;

use crate::support_api::queries::event_kinds;

/// Règle « objet contient `text` → marquer lu », avec un identifiant choisi par le client
pub fn subject_rule(name: &str, text: &str, scope: RuleScope) -> Rule {
    let matcher = TextMatcher {
        op: TextOp::Contains,
        values: vec![text.to_string()],
    };
    Rule {
        id: RuleId::from_raw("client-id"),
        name: name.to_string(),
        enabled: true,
        scope,
        alternatives: vec![ConditionGroup {
            conditions: vec![Condition {
                negated: false,
                test: ConditionTest::Subject { matcher },
            }],
        }],
        actions: vec![RuleAction::MarkRead { read: true }],
        schema_version: 1,
    }
}

/// Règle limitée au compte qui déplace les messages vers la boîte donnée
pub fn move_rule(account: &AccountId, mailbox: &str) -> Rule {
    let mut rule = subject_rule("Classement", "Sujet", only(account));
    rule.actions = vec![RuleAction::MoveTo {
        mailbox: MailboxId::from_raw(mailbox),
    }];
    rule
}

/// Portée d'un seul compte
pub fn only(account: &AccountId) -> RuleScope {
    RuleScope::Accounts(vec![account.clone()])
}

/// Règles annoncées par `RuleChanged`, dans l'ordre d'émission
pub async fn changed_rules(engine: &Engine) -> Vec<String> {
    let kinds = event_kinds(engine).await;
    kinds
        .into_iter()
        .filter_map(|kind| match kind {
            EventKind::RuleChanged { rule } => Some(rule.as_str().to_string()),
            _ => None,
        })
        .collect()
}

/// Noms de toutes les variantes d'une énumération étiquetée `type`, lus dans l'erreur de serde
pub fn variant_names<T: DeserializeOwned + std::fmt::Debug>() -> Vec<String> {
    let error = serde_json::from_str::<T>(r#"{"type":"__aucune__"}"#)
        .unwrap_err()
        .to_string();
    let listed = error.split("expected one of ").nth(1).unwrap();
    let mut names: Vec<String> = listed
        .split('`')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect();
    names.sort();
    names
}
