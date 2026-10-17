//! Téléchargements explicites (12.2.4) : corps, pièces jointes, recherche serveur ; seul lien réseau de la lecture.

mod access;
mod atomic_write;
mod attachment;
mod body;
mod download;
mod expected;
mod ingest;
mod locate;
mod results;
mod server_search;
mod task;

pub use access::ImapAccess;
pub use attachment::save_attachment;
pub use body::fetch_message_body;
pub use results::{ServerSearchHit, ServerSearchResult, ServerSearchResults};
pub use server_search::search_server;
pub use task::FetchTask;
