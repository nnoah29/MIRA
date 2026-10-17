//! Fils de conversation (D-15) : clé de fil, rattachement, recalcul sans perte et lectures locales.

mod assign;
mod key;
mod read;
mod recompute;
mod subject;

pub use assign::assign_thread;
pub use key::thread_key;
pub use read::{get_thread, list_threads};
pub use recompute::recompute_threads;
pub use subject::{normalize_subject, strip_reply_prefixes};
