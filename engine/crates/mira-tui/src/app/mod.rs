//! Socle de l'application plein écran : restauration du terminal et erreurs.

pub mod error;
pub mod terminal_guard;
pub mod terminal_probe;

pub use error::TuiError;
pub use terminal_guard::TerminalGuard;
pub use terminal_probe::TerminalProbe;
