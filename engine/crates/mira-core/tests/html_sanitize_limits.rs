#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Bornes de l'assainissement HTML (AG-34) : un message hostile ne gèle pas le moteur.

use std::time::{Duration, Instant};

use mira_core::html::sanitize_html;

/// Construit un HTML de `depth` `<div>` imbriqués autour d'un texte
fn nested_divs(depth: usize) -> String {
    format!("{}secret{}", "<div>".repeat(depth), "</div>".repeat(depth))
}

/// Une imbrication démesurée est traitée en moins d'une seconde et sans balise active
#[test]
fn regression_deeply_nested_html_is_sanitized_quickly() {
    let started = Instant::now();
    let safe = sanitize_html(&nested_divs(100_000), false);
    assert!(started.elapsed() < Duration::from_secs(1));
    assert!(!safe.html.contains("<div>"));
}

/// Un HTML démesuré en taille est borné lui aussi
#[test]
fn regression_oversized_html_is_truncated_quickly() {
    let started = Instant::now();
    let safe = sanitize_html(&"a<script>x</script>".repeat(2_000_000), false);
    assert!(started.elapsed() < Duration::from_secs(2));
    assert!(!safe.html.contains("<script"));
}

/// Un message trop imbriqué reste lisible en texte, balises échappées
#[test]
fn regression_over_limit_html_keeps_its_text_escaped() {
    let safe = sanitize_html(&nested_divs(5_000), false);
    assert!(safe.html.contains("secret"));
    assert!(!safe.html.contains("<div>"));
}

/// Une imbrication ordinaire reste assainie normalement
#[test]
fn nested_html_within_limits_is_unchanged() {
    let safe = sanitize_html(&nested_divs(50), false);
    assert_eq!(safe.html, nested_divs(50));
}
