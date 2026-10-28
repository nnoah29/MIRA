//! Libellés système (R4) : détection des conversations « en attente » et « mentions », écritures manuelles.

pub mod automated;
pub mod classify;
mod facts;
mod manual;
pub mod me;
pub mod mention;
mod refresh;

pub use classify::{LABEL_HORIZON_MS, WAITING_AFTER_MS};
pub use manual::set_system_label;
pub use refresh::refresh_account_labels;
