#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Lecteur SSE : événements simples, fins de ligne, commentaires, champs et données multi-lignes.

use mira_core::ai::sse::{SseEvent, SseReader};

/// Un événement complet dans un seul morceau
#[test]
fn single_event_in_one_chunk() {
    let mut reader = SseReader::new();
    let events = reader.push(b"data: hello\n\n");
    let expected = SseEvent {
        event: None,
        data: "hello".to_string(),
    };
    assert_eq!(events, vec![expected]);
}

/// Les fins de ligne CRLF sont acceptées et ne laissent aucun retour chariot dans les données
#[test]
fn crlf_line_endings() {
    let mut reader = SseReader::new();
    let events = reader.push(b"event: ping\r\ndata: one\r\n\r\ndata: two\r\n\r\n");
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].event.as_deref(), Some("ping"));
    assert_eq!(events[0].data, "one");
    assert_eq!(events[1].data, "two");
}

/// Les lignes de commentaire (keep-alive) sont ignorées et ne produisent aucun événement
#[test]
fn comment_lines_are_ignored() {
    let mut reader = SseReader::new();
    let events = reader.push(b": ping\n\n: ping\ndata: x\n\n");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data, "x");
}

/// Plusieurs lignes data sont jointes par un saut de ligne
#[test]
fn multi_line_data_joined_with_newline() {
    let mut reader = SseReader::new();
    let events = reader.push(b"data: first\ndata: second\ndata:\ndata: last\n\n");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data, "first\nsecond\n\nlast");
}

/// Le nom d'événement est conservé, et remis à zéro pour l'événement suivant
#[test]
fn event_name_is_kept() {
    let mut reader = SseReader::new();
    let events = reader.push(b"event: delta\ndata: a\n\ndata: b\n\n");
    assert_eq!(events[0].event.as_deref(), Some("delta"));
    assert_eq!(events[1].event, None);
}

/// Une seule espace après les deux-points est retirée, et seulement une
#[test]
fn data_without_space_after_colon() {
    let mut reader = SseReader::new();
    let events = reader.push(b"data:tight\n\ndata:  wide\n\n");
    assert_eq!(events[0].data, "tight");
    assert_eq!(events[1].data, " wide");
}
