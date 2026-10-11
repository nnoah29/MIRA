//! Lectures locales (invariant 27) : listes par vue, messages, corps, sources, pièces jointes, recherche.
//! Rien ici n'ouvre de connexion : tout passe par la base du profil.

mod attachments;
mod body;
mod counters;
mod list;
mod message;
mod query;
mod search;

pub use attachments::list_attachments;
pub use body::{BodyView, get_body, get_raw_source};
pub use counters::{mailboxes_with_counters, refresh_account_counters};
pub use list::list_messages;
pub use message::get_message;
pub use query::{ListQuery, MessageFilters, SearchFilters};
pub use search::search;
