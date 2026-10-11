#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Formulaires urlencoded : décodage, encodage et chaînes de requête.

use mira_testkit::oauth::form::{Form, build_query, decode, encode};

/// decode traduit les séquences %XX et le plus en espace.
#[test]
fn decode_handles_escapes_and_plus() {
    assert_eq!(decode("a%20b+c%3D%26"), "a b c=&");
    assert_eq!(decode("%C3%A9"), "é");
}

/// Une séquence invalide ou tronquée devient un point d'interrogation sans panique.
#[test]
fn decode_tolerates_invalid_escapes() {
    assert_eq!(decode("100%"), "100?");
    assert_eq!(decode("%zz"), "?");
}

/// encode puis decode redonne le texte d'origine.
#[test]
fn encode_then_decode_round_trips() {
    let original = "é &=%+/?#~-._ texte";
    assert_eq!(decode(&encode(original)), original);
    assert_eq!(encode("a b"), "a%20b");
}

/// parse découpe les paires, décode les valeurs et garde la première occurrence.
#[test]
fn parse_decodes_pairs_and_keeps_first_occurrence() {
    let form = Form::parse("a=1&b=x%20y&a=2&flag&=v");
    assert_eq!(form.get("a"), Some("1"));
    assert_eq!(form.get("b"), Some("x y"));
    assert_eq!(form.get("flag"), Some(""));
    assert_eq!(form.get("absent"), None);
}

/// build_query encode les noms et valeurs et se relit avec parse.
#[test]
fn build_query_round_trips_through_parse() {
    let query = build_query(&[("state", "a b&c"), ("code", "x=y")]);
    assert_eq!(query, "state=a%20b%26c&code=x%3Dy");
    let form = Form::parse(&query);
    assert_eq!(form.get("state"), Some("a b&c"));
    assert_eq!(form.get("code"), Some("x=y"));
}
