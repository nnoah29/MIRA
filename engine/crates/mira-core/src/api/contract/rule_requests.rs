//! Requêtes du domaine « règles » (spec R1 §7) : lecture, écritures idempotentes, validation, aperçu, catalogue.

use mira_vocabulary::{IdempotencyKey, Rule, RuleId};
use serde::{Deserialize, Serialize};

/// Opérations sur les règles ; à la création, l'identifiant fourni est remplacé par celui du moteur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum RuleRequest {
    ListRules,
    GetRule {
        rule: RuleId,
    },
    CreateRule {
        rule: Rule,
        idempotency_key: IdempotencyKey,
    },
    UpdateRule {
        rule: Rule,
        idempotency_key: IdempotencyKey,
    },
    DeleteRule {
        rule: RuleId,
        idempotency_key: IdempotencyKey,
    },
    SetRuleEnabled {
        rule: RuleId,
        enabled: bool,
        idempotency_key: IdempotencyKey,
    },
    ReorderRules {
        order: Vec<RuleId>,
        idempotency_key: IdempotencyKey,
    },
    ValidateRule {
        rule: Rule,
    },
    PreviewRule {
        rule: Rule,
        limit: u32,
    },
    ApplyRuleToExisting {
        rule: RuleId,
        idempotency_key: IdempotencyKey,
    },
    ListRuleCatalog,
    ExportRules,
    ImportRules {
        yaml: String,
        replace_existing: bool,
        idempotency_key: IdempotencyKey,
    },
}
