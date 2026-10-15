//! Intentions de la palette de commandes.

/// Ce que l'utilisateur fait avec la palette `:`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteIntent {
    Open,
    Char(char),
    Backspace,
    Submit,
    Close,
}
