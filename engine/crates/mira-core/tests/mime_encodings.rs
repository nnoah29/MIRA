#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Analyse MIME des encodages : quoted-printable, base64, ISO-8859-1, RFC 2047.

use mira_core::mime::parse_message;

/// Le quoted-printable est décodé en UTF-8
#[test]
fn quoted_printable_body_is_decoded() {
    let raw = b"Subject: qp\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: quoted-printable\r\n\r\nCaf=C3=A9 cr=C3=A8me=\r\n suite\r\n";
    let parsed = parse_message(raw);
    assert!(
        parsed.text_body.contains("Café crème suite"),
        "{:?}",
        parsed.text_body
    );
}

/// Un corps base64 est décodé
#[test]
fn base64_body_is_decoded() {
    let raw = b"Subject: b64\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Transfer-Encoding: base64\r\n\r\nQm9uam91ciDDoCB0b3VzIQ==\r\n";
    let parsed = parse_message(raw);
    assert_eq!(parsed.text_body.trim(), "Bonjour à tous!");
}

/// Un corps ISO-8859-1 est converti en UTF-8
#[test]
fn latin1_body_is_converted() {
    let mut raw = b"Subject: latin\r\nContent-Type: text/plain; charset=iso-8859-1\r\nContent-Transfer-Encoding: 8bit\r\n\r\nd".to_vec();
    raw.extend_from_slice(&[0xE9, 0xE8, b'\r', b'\n']);
    let parsed = parse_message(&raw);
    assert!(parsed.text_body.contains("déè"), "{:?}", parsed.text_body);
}

/// Les en-têtes RFC 2047 (mots encodés) sont décodés en UTF-8
#[test]
fn rfc2047_headers_are_decoded() {
    let raw = b"From: =?UTF-8?Q?Andr=C3=A9_Dupont?= <andre@example.org>\r\nSubject: =?UTF-8?B?w4l0w6kgMjAyNA==?=\r\n\r\ncorps\r\n";
    let parsed = parse_message(raw);
    assert_eq!(parsed.headers.subject, "Été 2024");
    assert_eq!(parsed.headers.from[0].name.as_deref(), Some("André Dupont"));
}

/// Un message HTML seul donne aussi un corps texte lisible sans balises
#[test]
fn html_only_message_has_plain_text() {
    let raw = b"Subject: h\r\nContent-Type: text/html; charset=utf-8\r\n\r\n<html><style>p{color:red}</style><body><p>Bonjour &amp; bienvenue</p><p>Suite</p></body></html>\r\n";
    let parsed = parse_message(raw);
    assert!(parsed.html_body.is_some());
    assert!(
        parsed.text_body.contains("Bonjour & bienvenue"),
        "{:?}",
        parsed.text_body
    );
    assert!(
        !parsed.text_body.contains("color"),
        "{:?}",
        parsed.text_body
    );
}
