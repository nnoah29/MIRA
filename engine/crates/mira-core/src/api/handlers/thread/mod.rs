//! Domaine « fils de conversation » : lectures locales.

mod get;
mod list;

use mira_vocabulary::MiraError;

use crate::api::contract::{Response, ThreadRequest};
use crate::api::engine::Engine;

/// Route une requête du domaine fils
pub(crate) async fn route(engine: &Engine, request: ThreadRequest) -> Result<Response, MiraError> {
    match request {
        ThreadRequest::ListThreads { mailbox, page } => {
            list::handle_list_threads(engine, &mailbox, &page)
        }
        ThreadRequest::GetThread { thread } => get::handle_get_thread(engine, &thread),
    }
}
