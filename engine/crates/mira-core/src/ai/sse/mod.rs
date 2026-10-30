//! Lecteur SSE incrémental : octets arbitrairement découpés en entrée, événements complets en sortie.

mod event;
mod line;

pub use event::SseEvent;

use event::EventBuilder;
use line::LineBuffer;

/// Lecteur à état : reçoit les morceaux du flux et rend les événements terminés.
#[derive(Debug, Default)]
pub struct SseReader {
    lines: LineBuffer,
    builder: EventBuilder,
}

impl SseReader {
    /// Crée un lecteur vide.
    pub fn new() -> Self {
        Self::default()
    }

    /// Ajoute un morceau d'octets et rend les événements qu'il termine.
    pub fn push(&mut self, bytes: &[u8]) -> Vec<SseEvent> {
        let lines = self.lines.push(bytes);
        lines
            .iter()
            .filter_map(|line| self.builder.feed_line(line))
            .collect()
    }

    /// Signale la fin du flux et rend l'événement resté sans ligne vide finale.
    pub fn finish(&mut self) -> Vec<SseEvent> {
        let mut events = Vec::new();
        if let Some(rest) = self.lines.take_rest() {
            events.extend(self.builder.feed_line(&rest));
        }
        events.extend(self.builder.feed_line(""));
        events
    }
}
