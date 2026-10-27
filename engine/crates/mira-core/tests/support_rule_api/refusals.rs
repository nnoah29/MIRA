//! Fichiers d'import dont chaque règle est refusée pour une raison différente.

use mira_vocabulary::{
    RuleImportOutcome as Outcome, RuleImportRefusal as Refusal, RuleImportReport,
};
use serde_json::{Value, json};

use super::portability::{outcomes, variant};

/// Règle exportée, copiée sous ce nom avec ce champ remplacé
fn broken(base: &Value, name: &str, field: &str, new: Value) -> Value {
    let mut copy = variant(base, &format!("id-{name}"), name);
    copy[field] = new;
    copy
}

/// Cinq règles cassées, une par cause de refus
pub fn refusal_items(base: &Value) -> Vec<Value> {
    let scope = json!({"type": "accounts", "accounts": ["x@y.fr"]});
    let mailbox = json!({"account": crate::support_write::USER, "name": "Nulle"});
    let move_to = json!([{"type": "move_to", "mailbox": mailbox}]);
    let label = json!([{"type": "add_local_label", "label": "?"}]);
    let template = json!([{"type": "reply_with_template", "template": "?"}]);
    vec![
        broken(base, "Compte", "scope", scope),
        broken(base, "Boîte", "actions", move_to),
        broken(base, "Libellé", "actions", label),
        broken(base, "Modèle", "actions", template),
        broken(base, "Vide", "actions", json!([])),
    ]
}

/// Raisons de refus du rapport, dans l'ordre ; échec si une règle n'est pas refusée
pub fn reasons_of(report: &RuleImportReport) -> Vec<Refusal> {
    outcomes(report)
        .into_iter()
        .map(|outcome| match outcome {
            Outcome::Refused { reason } => reason,
            other => panic!("attendu un refus : {other:?}"),
        })
        .collect()
}

/// Vérifie que le rapport nomme chaque cause de refus à la bonne position, avec le bon nom
pub fn assert_refusal_report(report: &RuleImportReport) {
    let reasons = reasons_of(report);
    let expected = [
        Refusal::UnknownAccount,
        Refusal::UnknownMailbox,
        Refusal::UnknownLabel,
        Refusal::UnknownTemplate,
    ];
    assert_eq!(&reasons[..4], expected);
    assert!(matches!(reasons[4], Refusal::InvalidRule { .. }));
    let seen: Vec<_> = report
        .entries
        .iter()
        .map(|e| (e.position, e.name.as_str()))
        .collect();
    let names = [
        (1, "Compte"),
        (2, "Boîte"),
        (3, "Libellé"),
        (4, "Modèle"),
        (5, "Vide"),
    ];
    assert_eq!(seen, names);
}
