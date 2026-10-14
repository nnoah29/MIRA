//! Lecture d'un fil et de ses messages (lecture locale).

use mira_vocabulary::{MiraError, ThreadId};

use crate::api::contract::{Response, ThreadView};
use crate::api::engine::Engine;
use crate::threads::get_thread;

/// Rend le fil et ses messages, du plus ancien au plus récent
pub(crate) fn handle_get_thread(engine: &Engine, thread: &ThreadId) -> Result<Response, MiraError> {
    let (thread, messages) = get_thread(&engine.database, thread)?;
    Ok(Response::Thread(ThreadView { thread, messages }))
}
