//! Événement SSE et accumulation des champs d'une ligne à l'autre, selon la norme.

/// Événement SSE complet : nom optionnel et données (lignes `data:` jointes par `\n`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SseEvent {
    pub event: Option<String>,
    pub data: String,
}

/// Champs de l'événement en cours de lecture.
#[derive(Debug, Default)]
pub(super) struct EventBuilder {
    event: Option<String>,
    data: Vec<String>,
}

impl EventBuilder {
    /// Consomme une ligne ; une ligne vide termine l'événement en cours.
    pub(super) fn feed_line(&mut self, line: &str) -> Option<SseEvent> {
        if line.is_empty() {
            return self.dispatch();
        }
        if line.starts_with(':') {
            return None;
        }
        let (field, value) = split_field(line);
        self.apply_field(field, value);
        None
    }

    /// Mémorise un champ connu ; les autres champs sont ignorés.
    fn apply_field(&mut self, field: &str, value: &str) {
        match field {
            "event" => self.event = Some(value.to_string()),
            "data" => self.data.push(value.to_string()),
            _ => {}
        }
    }

    /// Émet l'événement accumulé s'il porte des données, puis repart de zéro.
    fn dispatch(&mut self) -> Option<SseEvent> {
        let event = self.event.take();
        let lines = std::mem::take(&mut self.data);
        if lines.is_empty() {
            return None;
        }
        Some(SseEvent {
            event,
            data: lines.join("\n"),
        })
    }
}

/// Sépare `champ: valeur` en retirant au plus une espace après le deux-points.
fn split_field(line: &str) -> (&str, &str) {
    match line.split_once(':') {
        Some((field, value)) => (field, value.strip_prefix(' ').unwrap_or(value)),
        None => (line, ""),
    }
}
