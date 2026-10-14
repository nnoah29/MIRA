#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Conversion HTML vers texte : coût linéaire, balises masquées insensibles à la casse, plafond.

use std::time::{Duration, Instant};

use mira_core::mime::{html_to_text, parse_message};

/// 5 Mo de balises masquées vides se convertissent en moins d'une seconde
#[test]
fn regression_five_megabytes_of_style_tags_convert_quickly() {
    let hidden = "<style></style>".repeat(5 * 1024 * 1024 / 15);
    let started = Instant::now();
    let text = html_to_text(&format!("<p>debut</p>{hidden}<p>fin</p>"));
    assert!(
        started.elapsed() < Duration::from_secs(1),
        "{:?}",
        started.elapsed()
    );
    assert!(text.contains("debut"));
}

/// Les balises masquées en majuscules ou casse mixte sont retirées avec leur contenu
#[test]
fn hidden_tags_are_case_insensitive() {
    let text = html_to_text("<p>a</p><STYLE>x{}</Style><ScRiPt>alert(1)</SCRIPT><p>b</p>");
    assert!(text.contains('a') && text.contains('b'));
    assert!(!text.contains("alert") && !text.contains("x{}"));
}

/// Un HTML démesuré est plafonné : la conversion reste rapide et le début est conservé
#[test]
fn regression_oversized_html_is_capped_for_text_conversion() {
    let html = format!("<p>debut</p>{}", "<b>mot</b> ".repeat(2 * 1024 * 1024));
    let started = Instant::now();
    let text = html_to_text(&html);
    assert!(started.elapsed() < Duration::from_secs(2));
    assert!(text.starts_with("debut"));
    assert!(text.len() <= 2 * 1024 * 1024);
}

/// Un message HTML seul dont le corps est démesuré est analysé sans geler
#[test]
fn regression_parse_message_with_huge_style_body_is_fast() {
    let body = "<style></style>".repeat(4 * 1024 * 1024 / 15);
    let raw = format!("Subject: x\r\nContent-Type: text/html\r\n\r\n<p>salut</p>{body}");
    let started = Instant::now();
    let parsed = parse_message(raw.as_bytes());
    assert!(started.elapsed() < Duration::from_secs(2));
    assert!(parsed.text_body.contains("salut"));
}
