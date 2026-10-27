#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Conversion d'une règle vers sa forme portable (références en clair) et retour.

mod support_portability;

use mira_core::portability::rules::{from_portable, to_portable};
use mira_vocabulary::{ConditionTest, RuleAction};
use serde_json::json;
use support_portability::fake_refs::FakeRefs;
use support_portability::rule::{full_rule, portable, with_field};

/// Exporter puis importer redonne exactement la même règle
#[test]
fn round_trip_keeps_rule_identical() {
    let back = from_portable(portable(), &FakeRefs::default()).unwrap();
    assert_eq!(back, full_rule());
}

/// La forme portable ne contient que des adresses et des noms, aucun identifiant local
#[test]
fn portable_form_uses_names_not_ids() {
    let text = portable().to_string();
    for local in ["\"a1\"", "\"m1\"", "\"l1\"", "\"t1\""] {
        assert!(!text.contains(local), "{local} dans {text}");
    }
    for clear in [
        "moi@x.fr",
        "Archives/Factures",
        "Compta",
        "Accusé de réception",
    ] {
        assert!(text.contains(clear), "{clear} absent de {text}");
    }
}

/// Un libellé disparu fait échouer la conversion vers la forme portable
#[test]
fn export_of_dangling_label_fails_typed() {
    let refs = FakeRefs {
        without_label: true,
    };
    assert!(to_portable(&full_rule(), &refs).is_err());
}

/// Un test de condition sans référence garde sa forme
#[test]
fn conditions_without_references_are_untouched() {
    let rule = from_portable(portable(), &FakeRefs::default()).unwrap();
    assert!(matches!(
        rule.alternatives[0].conditions[0].test,
        ConditionTest::Subject { .. }
    ));
}

/// Les noms se retrouvent sans tenir compte de la casse ni des espaces de bord
#[test]
fn name_lookup_ignores_case_and_edge_spaces() {
    let actions = json!([{"type": "add_local_label", "label": "  COMPTA "}]);
    let value = with_field(portable(), "actions", actions);
    let rule = from_portable(value, &FakeRefs::default()).unwrap();
    assert!(
        matches!(&rule.actions[0], RuleAction::AddLocalLabel { label } if label.as_str() == "l1")
    );
}
