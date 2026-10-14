//! Plage locale (D-21) : « remonter plus loin » en tâche longue annulable.

mod extend;
mod spawn;

pub use extend::{ExtendOutcome, extend_window};
pub use spawn::start_extend_window;
