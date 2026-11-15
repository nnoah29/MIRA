#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Encodage des vecteurs de souvenirs en octets f32 little-endian et décodage contrôlé.

use mira_core::ai::memory::{decode_vector, encode_vector};

/// Encoder puis décoder rend les mêmes valeurs
#[test]
fn encode_decode_round_trip() {
    let values = [0.5_f32, -1.25, 3.0];
    let bytes = encode_vector(&values);
    assert_eq!(bytes.len(), 12);
    assert_eq!(decode_vector(&bytes, 3).unwrap(), values);
}

/// Une longueur qui ne vaut pas quatre fois la dimension est refusée
#[test]
fn decode_rejects_wrong_length() {
    assert_eq!(decode_vector(&[0; 11], 3), None);
    assert_eq!(decode_vector(&[0; 12], 2), None);
}
