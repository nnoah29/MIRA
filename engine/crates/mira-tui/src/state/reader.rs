//! Défilement du lecteur de message.

/// Position de lecture dans le message ouvert.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ReaderState {
    pub scroll: u16,
}
