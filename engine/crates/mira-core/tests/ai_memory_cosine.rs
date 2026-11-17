#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Similarité cosinus entre vecteurs de souvenirs : cas limites compris.

use mira_core::ai::memory::cosine;

/// Deux vecteurs identiques ont une similarité de 1
#[test]
fn cosine_of_identical_vectors_is_one() {
    let score = cosine(&[1.0, 2.0, 3.0], &[1.0, 2.0, 3.0]).unwrap();
    assert!((score - 1.0).abs() < 1e-6);
}

/// Deux vecteurs orthogonaux ont une similarité de 0
#[test]
fn cosine_of_orthogonal_vectors_is_zero() {
    let score = cosine(&[1.0, 0.0], &[0.0, 1.0]).unwrap();
    assert!(score.abs() < 1e-6);
}

/// Des dimensions différentes ne se comparent pas
#[test]
fn cosine_rejects_dimension_mismatch() {
    assert_eq!(cosine(&[1.0, 0.0], &[1.0, 0.0, 0.0]), None);
}

/// Un vecteur de norme nulle ne se compare pas
#[test]
fn cosine_rejects_zero_norm() {
    assert_eq!(cosine(&[0.0, 0.0], &[1.0, 0.0]), None);
}
