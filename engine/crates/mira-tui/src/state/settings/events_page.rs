//! Rangement d'une page reçue du journal et numéros qui servent à lire la suivante ou la précédente.

use mira_assembly::mira_vocabulary::{Event, EventSeq};

use super::events::{EVENTS_PAGE, EventsState, PageMove, PendingPage};

impl EventsState {
    /// Numéro après lequel lire la page suivante : le dernier de la page affichée.
    pub fn next_after(&self) -> Option<EventSeq> {
        self.entries.last().map(|event| event.seq)
    }

    /// Numéro après lequel avait été lue la page précédente, s'il y en a une.
    pub fn previous_after(&self) -> Option<Option<EventSeq>> {
        self.history.last().copied()
    }

    /// Range la page lue ; vrai s'il faut lire la suivante (recherche de la fin du journal).
    pub fn apply(&mut self, page: PendingPage, events: Vec<Event>) -> bool {
        let full = events.len() >= EVENTS_PAGE as usize;
        if events.is_empty() && page.movement == PageMove::Next {
            self.at_end = true;
            return false;
        }
        match page.movement {
            PageMove::First => self.history.clear(),
            PageMove::Next => self.history.push(self.after),
            PageMove::Previous => {
                self.history.pop();
            }
        }
        self.after = page.after;
        self.entries = events;
        self.loaded = true;
        self.at_end = !full;
        let to_bottom = page.seeking_end || page.movement == PageMove::Previous;
        self.select_edge(to_bottom);
        page.seeking_end && full
    }

    /// Sélectionne la première ou la dernière ligne de la page.
    fn select_edge(&mut self, bottom: bool) {
        let len = self.entries.len();
        if bottom {
            self.selection.bottom(len);
        } else {
            self.selection.top(len);
        }
    }
}
