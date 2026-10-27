#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Libellé local sous forme portable : aller-retour, couleur facultative, champs refusés.

use mira_core::portability::labels::{
    PortableLabel, PortableLabelError, from_portable, to_portable,
};
use mira_vocabulary::{LocalLabel, LocalLabelId};
use serde_json::json;

/// Exporter puis relire garde le nom et la couleur
#[test]
fn round_trip_keeps_name_and_color() {
    let label = LocalLabel {
        id: LocalLabelId::from_raw("l1"),
        name: "Compta".to_string(),
        color: 0xFF1E_90FF,
    };
    let back = from_portable(to_portable(&label)).unwrap();
    assert_eq!(
        back,
        PortableLabel {
            name: "Compta".to_string(),
            color: Some(0xFF1E_90FF)
        }
    );
}

/// La forme portable ne porte aucun identifiant local
#[test]
fn portable_form_has_no_id() {
    let label = LocalLabel {
        id: LocalLabelId::from_raw("l1"),
        name: "Compta".to_string(),
        color: 0xFF1E_90FF,
    };
    let text = to_portable(&label).to_string();
    assert!(!text.contains("l1") && !text.contains("id"), "{text}");
}

/// Sans couleur, le libellé est lu avec `None`
#[test]
fn color_is_optional() {
    let label = from_portable(json!({"name": "Voyages"})).unwrap();
    assert_eq!(
        label,
        PortableLabel {
            name: "Voyages".to_string(),
            color: None
        }
    );
}

/// Une couleur mal formée ou non textuelle est signalée à part
#[test]
fn invalid_color_is_reported_separately() {
    for color in [json!("rouge"), json!(12), json!("#FFF"), json!(null)] {
        let error = from_portable(json!({"name": "A", "color": color})).unwrap_err();
        assert_eq!(error, PortableLabelError::InvalidColor);
    }
}

/// Un champ inconnu rend le libellé invalide
#[test]
fn unknown_field_is_invalid() {
    let error = from_portable(json!({"name": "A", "couleur": "#ff0000"})).unwrap_err();
    assert_eq!(error, PortableLabelError::Invalid);
}

/// Pas de nom textuel : libellé invalide
#[test]
fn missing_or_non_text_name_is_invalid() {
    for value in [json!({}), json!({"name": 3}), json!("texte"), json!([1])] {
        assert_eq!(
            from_portable(value).unwrap_err(),
            PortableLabelError::Invalid
        );
    }
}
