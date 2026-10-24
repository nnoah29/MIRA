#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Contrat des événements : un exemple par variante de `EventKind`, relu par l'app ; l'instantané est la forme exacte sur le fil.

use std::collections::BTreeSet;

use mira_vocabulary::{Event, EventKind};
use serde_json::Value;

const KIND_SOURCE: &str = include_str!("../../mira-vocabulary/src/events/kind.rs");
const SNAPSHOT: &str = include_str!("snapshots/events_v1.json");

/// Instantané versionné des événements
fn snapshot() -> Value {
    serde_json::from_str(SNAPSHOT).unwrap()
}

/// Noms `snake_case` des variantes de `EventKind`, lus dans leur déclaration
fn declared_types() -> BTreeSet<String> {
    let start = KIND_SOURCE.find("pub enum EventKind {").unwrap();
    let body = &KIND_SOURCE[start..];
    let body = &body[..body.find("\n}\n").unwrap()];
    let variants = body.lines().filter_map(|line| {
        let name = line.strip_prefix("    ")?;
        let first = name.chars().next()?;
        let word: String = name.chars().take_while(|c| c.is_alphanumeric()).collect();
        (first.is_ascii_uppercase() && !word.is_empty()).then_some(word)
    });
    variants.map(|word| snake_case(&word)).collect()
}

/// `CamelCase` en `snake_case`
fn snake_case(word: &str) -> String {
    let mut out = String::new();
    for (index, c) in word.chars().enumerate() {
        if c.is_ascii_uppercase() && index > 0 {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

/// L'instantané nomme exactement les variantes de `EventKind`
#[test]
fn snapshot_names_exactly_the_declared_events() {
    let listed: BTreeSet<String> = snapshot()["events"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    assert!(listed.len() > 40);
    assert_eq!(listed, declared_types());
}

/// Chaque exemple est lu par le moteur puis réécrit à l'identique : c'est bien la forme exacte sur le fil
#[test]
fn every_example_is_the_exact_wire_form() {
    for (name, example) in snapshot()["events"].as_object().unwrap() {
        let kind: EventKind = serde_json::from_value(example.clone()).unwrap();
        assert_eq!(&serde_json::to_value(&kind).unwrap(), example, "{name}");
        assert_eq!(example["type"], Value::String(name.clone()));
    }
}

/// L'enveloppe numérotée et horodatée a la forme que l'app lit : numéro, instant en millisecondes, nature
#[test]
fn envelope_has_the_shape_the_app_reads() {
    let example = snapshot()["envelope"].clone();
    let event: Event = serde_json::from_value(example.clone()).unwrap();
    assert_eq!(serde_json::to_value(&event).unwrap(), example);
    assert_eq!(example["seq"], 42);
    assert!(example["kind"]["type"].is_string());
}
