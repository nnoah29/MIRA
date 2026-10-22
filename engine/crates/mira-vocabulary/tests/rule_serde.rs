#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

mod rule_samples;

use mira_vocabulary::{
    AccountId, Condition, ConditionGroup, ConditionTest, Rule, RuleAction, RuleId, RuleScope,
    TextMatcher, TextOp,
};

/// Une règle utilisant chaque variante survit à un aller-retour JSON.
#[test]
fn rule_roundtrip_keeps_all_variants() {
    let conditions = rule_samples::every_condition_test()
        .into_iter()
        .map(|test| Condition {
            negated: true,
            test,
        })
        .collect();
    let rule = Rule {
        id: RuleId::from_raw("rule-1"),
        name: "Tout".to_string(),
        enabled: true,
        scope: RuleScope::Accounts(vec![AccountId::from_raw("acc-1")]),
        alternatives: vec![ConditionGroup { conditions }],
        actions: rule_samples::every_rule_action(),
        schema_version: 1,
    };
    let json = serde_json::to_string(&rule).unwrap();
    let back: Rule = serde_json::from_str(&json).unwrap();
    assert_eq!(back, rule);
}

/// Les variantes sont étiquetées par `type` en snake_case.
#[test]
fn rule_json_shape_is_tagged() {
    let matcher = TextMatcher {
        op: TextOp::Contains,
        values: vec!["x".to_string()],
    };
    let value = serde_json::to_value(ConditionTest::Subject { matcher }).unwrap();
    assert_eq!(value["type"], "subject");
    assert_eq!(value["matcher"]["op"], "contains");
    let archive = serde_json::to_value(RuleAction::Archive).unwrap();
    assert_eq!(archive, serde_json::json!({"type": "archive"}));
    let scope = serde_json::to_value(RuleScope::AllAccounts).unwrap();
    assert_eq!(scope, serde_json::json!({"type": "all_accounts"}));
}

/// Une règle sans `schema_version` est lue en version 1.
#[test]
fn schema_version_defaults_to_one() {
    let json = serde_json::json!({
        "id": "r", "name": "n", "enabled": false,
        "scope": {"type": "all_accounts"},
        "alternatives": [], "actions": []
    });
    let rule: Rule = serde_json::from_value(json).unwrap();
    assert_eq!(rule.schema_version, 1);
}
