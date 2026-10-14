//! Application plein écran : boucle, pont asynchrone, clavier, restauration du terminal et erreurs.

pub mod engine_session;
pub mod error;
pub mod event_cursor;
pub mod event_feed;
pub mod input_thread;
pub mod panic_hook;
pub mod raw_mode;
pub mod request_task;
pub mod run_loop;
pub(crate) mod runtime;
pub(crate) mod screen;
pub mod session;
pub mod session_nonce;
pub mod stop_signals;
pub(crate) mod task_failure;
pub mod terminal_guard;

pub use engine_session::{close_engine, open_engine};
pub use error::TuiError;
pub use raw_mode::{RawMode, SystemRawMode};
pub use terminal_guard::TerminalGuard;
