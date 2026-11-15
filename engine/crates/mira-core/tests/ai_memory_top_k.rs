#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Classement des souvenirs les plus proches d'une requête : ordre, coupe et égalités.

use mira_core::ai::memory::top_k;

/// Les meilleurs d'abord, au plus `k`, sous le seuil écartés
#[test]
fn top_k_orders_and_cuts() {
    let items = vec![
        ("d".to_string(), vec![-1.0, 0.0]),
        ("b".to_string(), vec![1.0, 1.0]),
        ("c".to_string(), vec![0.0, 1.0]),
        ("a".to_string(), vec![1.0, 0.0]),
    ];
    let hits = top_k(&[1.0, 0.0], items.clone(), 3, 0.5);
    let ids: Vec<&str> = hits.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(ids, ["a", "b"]);
    assert_eq!(top_k(&[1.0, 0.0], items, 1, 0.0).len(), 1);
}

/// À score égal, l'identifiant le plus petit passe devant
#[test]
fn top_k_ties_break_by_id() {
    let items = vec![
        ("b".to_string(), vec![1.0, 0.0]),
        ("a".to_string(), vec![1.0, 0.0]),
    ];
    let hits = top_k(&[1.0, 0.0], items, 2, 0.0);
    let ids: Vec<&str> = hits.iter().map(|(id, _)| id.as_str()).collect();
    assert_eq!(ids, ["a", "b"]);
}
