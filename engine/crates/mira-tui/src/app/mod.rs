//! Socle de l'application plein écran : restauration du terminal et erreurs.

pub mod engine_session;
pub mod error;
pub mod raw_mode;
pub mod terminal_guard;

pub use engine_session::{close_engine, open_engine};
pub use error::TuiError;
pub use raw_mode::{RawMode, SystemRawMode};
pub use terminal_guard::TerminalGuard;
