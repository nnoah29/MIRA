#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Forme JSON de la condition et des actions sur les libellés locaux.

use mira_vocabulary::{ConditionTest, LocalLabelId, RuleAction};
use serde_json::json;

/// La condition « a le libellé » est étiquetée `has_local_label` et survit à un aller-retour
#[test]
fn has_local_label_is_tagged_and_round_trips() {
    let test = ConditionTest::HasLocalLabel {
        label: LocalLabelId::from_raw("lab-1"),
    };
    let value = serde_json::to_value(&test).unwrap();
    assert_eq!(value, json!({"type": "has_local_label", "label": "lab-1"}));
    assert_eq!(
        serde_json::from_value::<ConditionTest>(value).unwrap(),
        test
    );
}

/// Les actions de libellé sont étiquetées `add_local_label` et `remove_local_label`
#[test]
fn label_actions_are_tagged_and_round_trip() {
    let label = LocalLabelId::from_raw("lab-1");
    let add = RuleAction::AddLocalLabel {
        label: label.clone(),
    };
    let remove = RuleAction::RemoveLocalLabel { label };
    let shown = serde_json::to_value([&add, &remove]).unwrap();
    assert_eq!(
        shown,
        json!([
            {"type": "add_local_label", "label": "lab-1"},
            {"type": "remove_local_label", "label": "lab-1"}
        ])
    );
    let back: Vec<RuleAction> = serde_json::from_value(shown).unwrap();
    assert_eq!(back, vec![add, remove]);
}
