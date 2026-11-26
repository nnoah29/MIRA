//! Analyse MIME tolérante : encapsule `mail-parser` et ne rend jamais d'erreur (12.7).

mod addresses;
mod attachment_name;
mod attachments;
mod bodies;
mod headers;
mod html_entities;
mod html_text;
mod ids;
mod message;
mod model;
mod preview;
mod summary;
mod unsubscribe;
mod unsubscribe_host;
mod unsubscribe_ranges;

pub use headers::{parse_headers, parse_unsubscribe_headers};
pub use html_text::html_to_text;
pub use message::{attachment_bytes, parse_message};
pub use model::{HeaderSummary, ParsedAttachment, ParsedMessage};
pub use preview::preview_text;
pub(crate) use unsubscribe_host::is_private_host;
