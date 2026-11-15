#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Résultat de recherche de la mémoire pour l'API : score en millièmes, sans flottant dans le contrat.

use mira_vocabulary::{MemoryHit, MemoryKind, MemoryMatch, MemoryRecord, MemorySource, Timestamp};

/// Un souvenir d'exemple
fn record() -> MemoryRecord {
    MemoryRecord {
        id: "m".into(),
        text: "t".into(),
        kind: MemoryKind::Fact,
        source: MemorySource::User,
        created_at: Timestamp::from_millis(1),
        model_id: None,
    }
}

/// Le score est arrondi vers zéro en millièmes, négatif compris
#[test]
fn memory_match_converts_a_hit_score_to_permille() {
    let high = MemoryHit {
        record: record(),
        score: 0.8744,
    };
    assert_eq!(MemoryMatch::from(high).score_permille, 874);
    let low = MemoryHit {
        record: record(),
        score: -0.2,
    };
    assert_eq!(MemoryMatch::from(low).score_permille, -200);
}

/// Un score hors de -1..1 est ramené dans la plage
#[test]
fn memory_match_clamps_the_score() {
    let over = MemoryHit {
        record: record(),
        score: 3.0,
    };
    assert_eq!(MemoryMatch::from(over).score_permille, 1000);
}
