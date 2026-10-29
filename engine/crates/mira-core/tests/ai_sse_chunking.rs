#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Lecteur SSE : robustesse aux découpages arbitraires, à l'UTF-8 coupé et à la fin de flux.

use mira_core::ai::sse::{SseEvent, SseReader};

/// Lit un flux en deux morceaux coupés à `cut`, puis le vide
fn read_split(stream: &[u8], cut: usize) -> Vec<SseEvent> {
    let mut reader = SseReader::new();
    let mut events = reader.push(&stream[..cut]);
    events.extend(reader.push(&stream[cut..]));
    events.extend(reader.finish());
    events
}

/// Le même flux coupé à chaque position d'octet donne exactement les mêmes événements
#[test]
fn event_split_at_every_byte_position() {
    let stream = "event: a\r\ndata: caf\u{e9} \u{1f600}\r\n: ping\r\ndata: l2\r\n\r\ndata: z\n\n";
    let whole = read_split(stream.as_bytes(), 0);
    assert_eq!(whole.len(), 2);
    for cut in 0..=stream.len() {
        assert_eq!(read_split(stream.as_bytes(), cut), whole, "cut={cut}");
    }
}

/// Un caractère accentué et un émoji coupés au milieu de leurs octets sont reconstitués
#[test]
fn utf8_character_split_across_chunks() {
    let mut reader = SseReader::new();
    let bytes = "data: \u{e9}\u{1f600}\n\n".as_bytes();
    let mut events = Vec::new();
    for byte in bytes {
        events.extend(reader.push(std::slice::from_ref(byte)));
    }
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data, "\u{e9}\u{1f600}");
}

/// Sans ligne vide finale, l'événement n'est rendu qu'à la fin du flux
#[test]
fn stream_ending_without_blank_line_is_flushed_by_finish() {
    let mut reader = SseReader::new();
    assert!(reader.push(b"data: tail").is_empty());
    let events = reader.finish();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].data, "tail");
    assert!(reader.finish().is_empty());
}

/// Un retour chariot seul termine une ligne, même quand le saut de ligne arrive au morceau suivant
#[test]
fn lone_carriage_return_ends_a_line() {
    let mut reader = SseReader::new();
    let mut events = reader.push(b"data: a\r\r");
    events.extend(reader.push(b"data: b\r"));
    events.extend(reader.push(b"\n\r\n"));
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].data, "a");
    assert_eq!(events[1].data, "b");
}
