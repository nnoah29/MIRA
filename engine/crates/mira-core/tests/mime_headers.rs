#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Analyse MIME des en-têtes seuls : adresses, identifiants, références, date.

use mira_core::mime::parse_headers;

const THREADED: &[u8] = b"From: \"Durand, Claire\" <claire@example.org>\r\nTo: a@example.org, B Bob <b@example.org>\r\nCc: c@example.org\r\nBcc: d@example.org\r\nReply-To: reponse@example.org\r\nSubject: Re: suite\r\nMessage-ID: <m3@mira>\r\nIn-Reply-To: <m2@mira>\r\nReferences: <m1@mira>\r\n <m2@mira>\r\n\r\n";

/// Les adresses sont séparées en listes, avec leur nom d'affichage
#[test]
fn addresses_are_split_by_field() {
    let summary = parse_headers(THREADED);
    assert_eq!(summary.from[0].address, "claire@example.org");
    assert_eq!(summary.from[0].name.as_deref(), Some("Durand, Claire"));
    assert_eq!(summary.to.len(), 2);
    assert_eq!(summary.to[1].name.as_deref(), Some("B Bob"));
    assert_eq!(summary.cc[0].address, "c@example.org");
    assert_eq!(summary.bcc[0].address, "d@example.org");
    assert_eq!(summary.reply_to[0].address, "reponse@example.org");
}

/// Message-ID, In-Reply-To et References sont rendus sans chevrons et dans l'ordre
#[test]
fn identifiers_have_no_angle_brackets() {
    let summary = parse_headers(THREADED);
    assert_eq!(summary.message_id.as_deref(), Some("m3@mira"));
    assert_eq!(summary.in_reply_to.as_deref(), Some("m2@mira"));
    assert_eq!(summary.references, vec!["m1@mira", "m2@mira"]);
    assert_eq!(summary.subject, "Re: suite");
}

/// Plusieurs En-têtes References sur des lignes séparées sont tous conservés
#[test]
fn repeated_references_headers_are_kept() {
    let raw = b"Subject: x\r\nReferences: <a@x>\r\nReferences: <b@x> <c@x>\r\n\r\n";
    let summary = parse_headers(raw);
    assert!(
        summary.references.contains(&"a@x".to_string()),
        "{summary:?}"
    );
    assert!(
        summary.references.contains(&"c@x".to_string()),
        "{summary:?}"
    );
}

/// Une date avec fuseau est convertie en millisecondes UTC, une date absurde est ignorée
#[test]
fn date_is_converted_to_utc_millis() {
    let zoned = parse_headers(b"Date: Tue, 14 Nov 2023 23:13:20 +0100\r\n\r\n");
    assert_eq!(zoned.date.unwrap().as_millis(), 1_700_000_000_000);
    let invalid = parse_headers(b"Date: pas une date\r\nSubject: s\r\n\r\n");
    assert!(invalid.date.is_none());
    assert_eq!(invalid.subject, "s");
}

/// Un groupe d'adresses est aplati et des en-têtes vides donnent un résumé vide
#[test]
fn groups_are_flattened_and_empty_input_is_empty() {
    let grouped = parse_headers(b"To: Equipe: a@x.org, b@x.org;\r\n\r\n");
    assert_eq!(grouped.to.len(), 2);
    let empty = parse_headers(b"");
    assert!(empty.from.is_empty() && empty.subject.is_empty() && empty.date.is_none());
}
