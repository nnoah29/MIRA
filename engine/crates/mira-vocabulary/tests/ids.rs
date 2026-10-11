#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use mira_vocabulary::{AccountId, EventSeq, MessageId, Timestamp};

/// Un identifiant se sérialise comme une chaîne JSON simple et se relit à l'identique.
#[test]
fn account_id_serializes_as_plain_string() {
    let id = AccountId::from_raw("acc-1");
    let json = serde_json::to_string(&id).unwrap();
    assert_eq!(json, "\"acc-1\"");
    let back: AccountId = serde_json::from_str(&json).unwrap();
    assert_eq!(back, id);
}

/// Deux identifiants de même valeur sont égaux et l'ordre suit la chaîne.
#[test]
fn ids_compare_by_value_and_order() {
    let a = MessageId::from_raw("a");
    let b = MessageId::from_raw("b");
    assert_eq!(a, MessageId::from_raw("a"));
    assert_ne!(a, b);
    assert!(a < b);
}

/// L'affichage et les accesseurs rendent la chaîne opaque telle quelle.
#[test]
fn id_display_and_accessors_return_raw_value() {
    let id = AccountId::from_raw("x-42");
    assert_eq!(id.to_string(), "x-42");
    assert_eq!(id.as_str(), "x-42");
    assert_eq!(id.into_string(), "x-42");
}

/// Un numéro d'événement et un instant sont ordonnés et l'instant se décale.
#[test]
fn event_seq_and_timestamp_are_ordered() {
    assert!(EventSeq(1) < EventSeq(2));
    let t = Timestamp::from_millis(1_000);
    assert_eq!(t.plus_millis(500).as_millis(), 1_500);
    assert!(t < t.plus_millis(1));
    assert_eq!(serde_json::to_string(&t).unwrap(), "1000");
}
