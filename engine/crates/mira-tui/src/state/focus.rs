//! Panneau qui reçoit le clavier.

/// Panneaux de l'écran principal, de gauche à droite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Sidebar,
    List,
    Reader,
}

impl Pane {
    /// Panneau suivant, en cycle.
    pub fn next(self) -> Pane {
        match self {
            Pane::Sidebar => Pane::List,
            Pane::List => Pane::Reader,
            Pane::Reader => Pane::Sidebar,
        }
    }

    /// Panneau précédent, en cycle.
    pub fn previous(self) -> Pane {
        match self {
            Pane::Sidebar => Pane::Reader,
            Pane::List => Pane::Sidebar,
            Pane::Reader => Pane::List,
        }
    }
}
