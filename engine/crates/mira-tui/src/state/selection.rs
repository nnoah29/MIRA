//! Sélection d'un élément dans une liste, toujours bornée par la longueur de la liste.

/// Index sélectionné ; `None` tant que la liste est vide.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Selection {
    pub selected: Option<usize>,
}

impl Selection {
    /// Descend d'un élément sans dépasser le dernier.
    pub fn down(&mut self, len: usize) {
        self.selected = match (len, self.selected) {
            (0, _) => None,
            (_, None) => Some(0),
            (_, Some(index)) => Some((index + 1).min(len - 1)),
        };
    }

    /// Monte d'un élément sans dépasser le premier.
    pub fn up(&mut self, len: usize) {
        self.selected = match (len, self.selected) {
            (0, _) => None,
            (_, None) => Some(0),
            (_, Some(index)) => Some(index.saturating_sub(1).min(len - 1)),
        };
    }

    /// Va au premier élément.
    pub fn top(&mut self, len: usize) {
        self.selected = if len == 0 { None } else { Some(0) };
    }

    /// Va au dernier élément.
    pub fn bottom(&mut self, len: usize) {
        self.selected = len.checked_sub(1);
    }
}
