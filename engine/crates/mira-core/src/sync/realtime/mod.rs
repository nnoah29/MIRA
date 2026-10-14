//! Temps réel (10.6) : IDLE sur la réception, relevé périodique en repli, reconnexion avec délai croissant.

mod backoff;
mod idle_watcher;

pub use backoff::{Backoff, BackoffPolicy};
pub use idle_watcher::{IdleSignal, IdleWatcher, start_idle_watcher};
