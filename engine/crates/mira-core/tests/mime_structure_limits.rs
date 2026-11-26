#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Analyse MIME : structures hostiles (milliers de parties, imbrication extrême) sans panique.

use mira_core::mime::parse_message;
use std::time::{Duration, Instant};

/// Un multipart de 10 000 parties est lu dans un délai raisonnable
#[test]
fn ten_thousand_parts_are_parsed_quickly() {
    let mut raw =
        String::from("Subject: x\r\nContent-Type: multipart/mixed; boundary=\"b\"\r\n\r\n");
    for _ in 0..10_000 {
        raw.push_str("--b\r\nContent-Type: text/plain\r\n\r\nx\r\n");
    }
    raw.push_str("--b--\r\n");
    let start = Instant::now();
    let parsed = parse_message(raw.as_bytes());
    assert_eq!(parsed.headers.subject, "x");
    assert!(start.elapsed() < Duration::from_secs(10));
}

/// Des multiparts imbriqués sur 5 000 niveaux ne dépassent pas la pile d'un thread modeste
#[test]
fn deeply_nested_multiparts_do_not_overflow_the_stack() {
    let depth = 5_000;
    let mut raw = String::from("Subject: x\r\n");
    for level in 0..depth {
        raw.push_str(&format!(
            "Content-Type: multipart/mixed; boundary=\"b{level}\"\r\n\r\n--b{level}\r\n"
        ));
    }
    raw.push_str("Content-Type: text/plain\r\n\r\nfin\r\n");
    let outcome = std::thread::Builder::new()
        .stack_size(512 * 1024)
        .spawn(move || parse_message(raw.as_bytes()).headers.subject)
        .unwrap()
        .join();
    assert!(outcome.is_ok());
}
