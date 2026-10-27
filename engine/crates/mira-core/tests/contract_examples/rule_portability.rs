//! Exemples d'export et d'import YAML des règles.

use mira_core::api::{Response, RuleRequest, RuleResponse};
use mira_vocabulary::{
    IdempotencyKey, RuleImportEntry, RuleImportOutcome, RuleImportRefusal, RuleImportReport,
};

use super::Named;

/// Les deux requêtes : export, et import avec remplacement
pub fn requests() -> Vec<RuleRequest> {
    vec![
        RuleRequest::ExportRules,
        RuleRequest::ImportRules {
            yaml: "mira_export:\n  kind: rules\n  version: 1\nrules: []\n".to_string(),
            replace_existing: false,
            idempotency_key: IdempotencyKey("key-1".to_string()),
        },
    ]
}

/// Rapport d'exemple : un sort de chaque sorte
fn sample_report() -> RuleImportReport {
    let entry = |position, name: &str, outcome| RuleImportEntry {
        position,
        name: name.to_string(),
        outcome,
    };
    let refusal = RuleImportRefusal::InvalidRule {
        code: "invalid_input.parameter".to_string(),
    };
    RuleImportReport {
        entries: vec![
            entry(1, "Lettres", RuleImportOutcome::Imported),
            entry(2, "Factures", RuleImportOutcome::Replaced),
            entry(3, "Alertes", RuleImportOutcome::AlreadyPresent),
            entry(4, "Cassée", RuleImportOutcome::Refused { reason: refusal }),
            entry(
                5,
                "Perdue",
                RuleImportOutcome::Refused {
                    reason: RuleImportRefusal::UnknownLabel,
                },
            ),
        ],
    }
}

/// Les deux réponses : texte exporté et rapport d'import
pub fn responses() -> Vec<Named<Response>> {
    let export = RuleResponse::RulesExport {
        yaml: "mira_export: {}\n".to_string(),
    };
    vec![
        ("rule.rules_export".to_string(), Response::Rule(export)),
        (
            "rule.rules_import".to_string(),
            Response::Rule(RuleResponse::RulesImport(sample_report())),
        ),
    ]
}
