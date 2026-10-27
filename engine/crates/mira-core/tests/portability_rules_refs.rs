#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Références d'une règle du fichier : chacune introuvable est refusée avec sa raison.

mod support_portability;

use mira_core::portability::rules::{PortableError, from_portable};
use serde_json::{Value, json};
use support_portability::fake_refs::FakeRefs;
use support_portability::rule::{portable, with_field};

/// Erreur rendue par la conversion du fichier dont un champ est remplacé
fn refused(field: &str, new: Value) -> PortableError {
    from_portable(with_field(portable(), field, new), &FakeRefs::default()).unwrap_err()
}

/// Compte inconnu dans la portée
#[test]
fn unknown_account_in_scope() {
    let scope = json!({"type": "accounts", "accounts": ["autre@x.fr"]});
    assert_eq!(refused("scope", scope), PortableError::UnknownAccount);
}

/// Compte inconnu dans une condition
#[test]
fn unknown_account_in_condition() {
    let test = json!({"type": "from_account", "account": "autre@x.fr"});
    let groups = json!([{"conditions": [{"negated": false, "test": test}]}]);
    assert_eq!(
        refused("alternatives", groups),
        PortableError::UnknownAccount
    );
}

/// Boîte inconnue dans une action
#[test]
fn unknown_mailbox() {
    let mailbox = json!({"account": "moi@x.fr", "name": "Nulle"});
    let actions = json!([{"type": "move_to", "mailbox": mailbox}]);
    assert_eq!(refused("actions", actions), PortableError::UnknownMailbox);
}

/// Libellé inconnu dans une action
#[test]
fn unknown_label() {
    let actions = json!([{"type": "add_local_label", "label": "Inconnu"}]);
    assert_eq!(refused("actions", actions), PortableError::UnknownLabel);
}

/// Modèle inconnu dans une action
#[test]
fn unknown_template() {
    let actions = json!([{"type": "reply_with_template", "template": "Inconnu"}]);
    assert_eq!(refused("actions", actions), PortableError::UnknownTemplate);
}
