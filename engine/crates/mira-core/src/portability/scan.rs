//! Contrôle d'un texte YAML avant lecture : alias et clés en double refusés, positions sans contenu.

use std::collections::HashSet;

use yaml_rust2::parser::{Event, MarkedEventReceiver, Parser};
use yaml_rust2::scanner::Marker;

/// Cause d'un refus : un alias, ou une clé répétée dans le même objet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Problem {
    Alias,
    DuplicateKey,
    Syntax,
}

/// Un objet ou une liste en cours de lecture : sa nature, le nombre de nœuds vus et ses clés
#[derive(Debug, Default)]
struct Frame {
    map: bool,
    seen: usize,
    keys: HashSet<String>,
}

/// Récepteur d'événements qui retient le premier alias ou la première clé en double
#[derive(Debug, Default)]
struct Guard {
    frames: Vec<Frame>,
    found: Option<(Problem, Marker)>,
}

impl Guard {
    /// Compte un nouveau nœud dans son parent ; une clé de texte déjà vue dans cet objet est un doublon
    fn note_node(&mut self, key: Option<String>, mark: Marker) {
        let Some(frame) = self.frames.last_mut() else {
            return;
        };
        let is_key = frame.map && frame.seen % 2 == 0;
        frame.seen += 1;
        let repeated = key.is_some_and(|text| is_key && !frame.keys.insert(text));
        if repeated && self.found.is_none() {
            self.found = Some((Problem::DuplicateKey, mark));
        }
    }
}

impl MarkedEventReceiver for Guard {
    /// Suit la structure du document et note ce qu'il faut refuser
    fn on_event(&mut self, event: Event, mark: Marker) {
        match event {
            Event::Alias(_) => {
                self.found.get_or_insert((Problem::Alias, mark));
            }
            Event::Scalar(text, ..) => self.note_node(Some(text), mark),
            Event::SequenceStart(..) | Event::MappingStart(..) => {
                self.note_node(None, mark);
                let map = matches!(event, Event::MappingStart(..));
                self.frames.push(Frame {
                    map,
                    ..Frame::default()
                });
            }
            Event::SequenceEnd | Event::MappingEnd => {
                self.frames.pop();
            }
            _ => {}
        }
    }
}

/// Parcourt le texte sans rien construire ; rend la cause du refus et sa position (ligne, colonne à partir de 1)
pub(super) fn scan(text: &str) -> Result<(), (Problem, usize, usize)> {
    let mut guard = Guard::default();
    let loaded = Parser::new_from_str(text).load(&mut guard, true);
    if let Some((problem, mark)) = guard.found {
        return Err((problem, mark.line(), mark.col() + 1));
    }
    loaded.map_err(|error| {
        (
            Problem::Syntax,
            error.marker().line(),
            error.marker().col() + 1,
        )
    })
}
