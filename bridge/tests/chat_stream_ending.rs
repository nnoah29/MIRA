#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Fin de flux sans `done` : une ligne terminale d'échec remplace le `done` manquant.

use mira_bridge::chat_feed::closing_line;
use serde_json::Value;

/// Sans `done` reçu et le client à l'écoute, une ligne `done` d'échec `server.ai_unavailable` est émise.
#[test]
fn stream_ending_without_done_emits_a_failed_done_line() {
    let line = closing_line(false, true).unwrap();
    let value: Value = serde_json::from_str(&line).unwrap();
    assert_eq!(value["type"], "done");
    assert_eq!(value["reason"]["failed"]["code"], "server.ai_unavailable");
}

/// Aucune ligne de secours si `done` est déjà passé ou si le flux Dart est fermé.
#[test]
fn no_closing_line_after_done_or_when_sink_is_closed() {
    assert_eq!(closing_line(true, true), None);
    assert_eq!(closing_line(false, false), None);
    assert_eq!(closing_line(true, false), None);
}
