//! Exemples du domaine « règles » : lecture, écritures idempotentes, validation, aperçu, catalogue.

use mira_core::api::{
    CatalogEntry, Request, Response, RuleCatalog, RulePreview, RuleRequest, RuleResponse, RuleView,
};
use mira_core::rules::validate::RuleStatus;
use mira_vocabulary::{
    AccountId, AddressField, Condition, ConditionGroup, ConditionTest, IdempotencyKey, MessageId,
    Rule, RuleAction, RuleId, RuleScope, TextMatcher, TextOp,
};

use super::{Named, numbered};

/// Règle d'exemple : expéditeur du domaine `example.org` sur un compte → marquer lu, ne pas notifier
pub fn sample_rule() -> Rule {
    let matcher = TextMatcher {
        op: TextOp::Equals,
        values: vec!["example.org".to_string()],
    };
    let test = ConditionTest::Sender {
        field: AddressField::Domain,
        matcher,
    };
    Rule {
        id: RuleId::from_raw("rule-1"),
        name: "Lettres".to_string(),
        enabled: true,
        scope: RuleScope::Accounts(vec![AccountId::from_raw("account-1")]),
        alternatives: vec![ConditionGroup {
            conditions: vec![Condition {
                negated: false,
                test,
            }],
        }],
        actions: vec![
            RuleAction::MarkRead { read: true },
            RuleAction::SuppressNotification,
        ],
        schema_version: 1,
    }
}

/// Une requête par écriture idempotente de `RuleRequest`
fn write_requests() -> Vec<RuleRequest> {
    let (rule, id) = (sample_rule(), RuleId::from_raw("rule-1"));
    let key = || IdempotencyKey("key-1".to_string());
    vec![
        RuleRequest::CreateRule {
            rule: rule.clone(),
            idempotency_key: key(),
        },
        RuleRequest::UpdateRule {
            rule: rule.clone(),
            idempotency_key: key(),
        },
        RuleRequest::DeleteRule {
            rule: id.clone(),
            idempotency_key: key(),
        },
        RuleRequest::SetRuleEnabled {
            rule: id.clone(),
            enabled: false,
            idempotency_key: key(),
        },
        RuleRequest::ReorderRules {
            order: vec![id],
            idempotency_key: key(),
        },
    ]
}

/// Une requête par opération de `RuleRequest`
pub fn requests() -> Vec<Named<Request>> {
    let mut items = vec![
        RuleRequest::ListRules,
        RuleRequest::GetRule {
            rule: RuleId::from_raw("rule-1"),
        },
    ];
    items.extend(write_requests());
    items.extend([
        RuleRequest::ValidateRule {
            rule: sample_rule(),
        },
        RuleRequest::PreviewRule {
            rule: sample_rule(),
            limit: 20,
        },
        RuleRequest::ListRuleCatalog,
    ]);
    let ops: Vec<&str> = items.iter().map(op_name).collect();
    let names = numbered("rule", &ops);
    names
        .into_iter()
        .zip(items.into_iter().map(Request::Rule))
        .collect()
}

/// Catalogue d'exemple réduit à une condition et une action
fn sample_catalog() -> RuleCatalog {
    let entry = |kind: &str, family: &str, fields: &[&str], mailbox: bool| CatalogEntry {
        kind: kind.to_string(),
        family: family.to_string(),
        fields: fields.iter().map(|f| f.to_string()).collect(),
        needs_mailbox: mailbox,
        single_account: mailbox,
    };
    RuleCatalog {
        conditions: vec![entry("sender", "sender", &["field", "matcher"], false)],
        actions: vec![entry("move_to", "filing", &["mailbox"], true)],
    }
}

/// Une réponse par forme de `RuleResponse`, dont un statut invalide
pub fn responses() -> Vec<Named<Response>> {
    let view = RuleView {
        rule: sample_rule(),
        status: RuleStatus::Valid,
    };
    let invalid = RuleStatus::Invalid {
        reason: "la portée de la règle est vide".to_string(),
    };
    let preview = RulePreview {
        matched: 2,
        scanned: 40,
        samples: vec![MessageId::from_raw("message-1")],
    };
    let items = [
        ("rule.rules", RuleResponse::Rules(vec![view.clone()])),
        ("rule.rule", RuleResponse::Rule(view)),
        ("rule.status", RuleResponse::Status(RuleStatus::Valid)),
        ("rule.status.2", RuleResponse::Status(invalid)),
        ("rule.preview", RuleResponse::Preview(preview)),
        ("rule.catalog", RuleResponse::Catalog(sample_catalog())),
    ];
    let wrap = |(name, inner): (&str, RuleResponse)| (name.to_string(), Response::Rule(inner));
    items.into_iter().map(wrap).collect()
}

/// Nom de l'opération, par `match` exhaustif
pub fn op_name(request: &RuleRequest) -> &'static str {
    match request {
        RuleRequest::ListRules => "list_rules",
        RuleRequest::GetRule { .. } => "get_rule",
        RuleRequest::CreateRule { .. } => "create_rule",
        RuleRequest::UpdateRule { .. } => "update_rule",
        RuleRequest::DeleteRule { .. } => "delete_rule",
        RuleRequest::SetRuleEnabled { .. } => "set_rule_enabled",
        RuleRequest::ReorderRules { .. } => "reorder_rules",
        RuleRequest::ValidateRule { .. } => "validate_rule",
        RuleRequest::PreviewRule { .. } => "preview_rule",
        RuleRequest::ListRuleCatalog => "list_rule_catalog",
    }
}
