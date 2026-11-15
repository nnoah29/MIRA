#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Types de la mémoire de l'IA : souvenirs, origine, genre et résultat de recherche.

use mira_vocabulary::{MemoryHit, MemoryKind, MemoryRecord, MemorySource, Timestamp};

/// Un souvenir d'exemple
fn sample() -> MemoryRecord {
    MemoryRecord {
        id: "mem-1".into(),
        text: "Je signe mes mails « Noah ».".into(),
        kind: MemoryKind::Preference,
        source: MemorySource::User,
        created_at: Timestamp::from_millis(1_000),
        model_id: Some("embed-1".into()),
    }
}

/// Le genre se sérialise en minuscules avec soulignés
#[test]
fn memory_kind_serializes_snake_case() {
    let json = serde_json::to_string(&MemoryKind::Preference).unwrap();
    assert_eq!(json, "\"preference\"");
}

/// L'origine se sérialise de la même façon
#[test]
fn memory_source_serializes_snake_case() {
    assert_eq!(serde_json::to_string(&MemorySource::Ai).unwrap(), "\"ai\"");
}

/// Un souvenir survit à un aller-retour JSON
#[test]
fn memory_record_round_trips() {
    let record = sample();
    let back: MemoryRecord =
        serde_json::from_str(&serde_json::to_string(&record).unwrap()).unwrap();
    assert_eq!(back, record);
}

/// Un résultat de recherche garde son score
#[test]
fn memory_hit_keeps_score() {
    let hit = MemoryHit {
        record: sample(),
        score: 0.75,
    };
    let back: MemoryHit = serde_json::from_str(&serde_json::to_string(&hit).unwrap()).unwrap();
    assert_eq!(back, hit);
}
