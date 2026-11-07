#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Vecteurs partagés avec le client Dart : les octets exacts de chaque type de trame.

use std::collections::BTreeSet;
use std::fmt::Write;

use mira_protocol::{Frame, MAX_FRAME_BYTES, encode};
use serde_json::Value;

/// Les vecteurs du fichier partagé : `{name, frame, hex}`.
fn vectors() -> Vec<Value> {
    serde_json::from_str(include_str!("golden/frames.json")).unwrap()
}

/// Octets en hexadécimal minuscule.
fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut out, byte| {
        let _ = write!(out, "{byte:02x}");
        out
    })
}

/// L'encodeur produit exactement les octets du fichier ; les écarts sont listés `nom=hex`.
#[test]
fn golden_vectors_match_the_encoder() {
    let mut mismatches = Vec::new();
    for vector in vectors() {
        let frame: Frame = serde_json::from_value(vector["frame"].clone()).unwrap();
        let actual = hex(&encode(&frame, MAX_FRAME_BYTES).unwrap());
        if vector["hex"] != actual.as_str() {
            mismatches.push(format!("{}={actual}", vector["name"].as_str().unwrap()));
        }
    }
    assert!(mismatches.is_empty(), "\n{}", mismatches.join("\n"));
}

/// Les dix-sept types de trame figurent tous dans les vecteurs.
#[test]
fn golden_vectors_cover_every_frame_type() {
    let seen: BTreeSet<String> = vectors()
        .iter()
        .map(|v| v["frame"]["type"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(seen.len(), 17, "{seen:?}");
}
