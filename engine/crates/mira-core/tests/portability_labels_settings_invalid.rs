#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Réglages d'un libellé local sous forme portable : valeurs refusées, champs inconnus.

use mira_core::portability::labels::{PortableLabelError, read_portable_entry};
use serde_json::json;

/// Une valeur de mauvais type ou un compteur inconnu rend l'entrée invalide
#[test]
fn wrong_setting_values_are_invalid() {
    for extra in [
        json!({"hidden": "oui"}),
        json!({"notifications": 1}),
        json!({"hide_from_inbox": null}),
        json!({"counter": "beaucoup"}),
        json!({"counter": 2}),
    ] {
        let mut value = json!({"name": "A"});
        value
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        let error = read_portable_entry(value).unwrap_err();
        assert_eq!(error, PortableLabelError::Invalid);
    }
}

/// Un champ inconnu reste refusé même quand des réglages sont présents
#[test]
fn unknown_field_stays_invalid_next_to_settings() {
    let value = json!({"name": "A", "hidden": true, "position": 2});
    assert_eq!(
        read_portable_entry(value).unwrap_err(),
        PortableLabelError::Invalid
    );
}
