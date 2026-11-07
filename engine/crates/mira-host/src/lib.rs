//! Hôte du moteur : logique commune au pont FFI et au démon.
//! Un `Host` possède le moteur ouvert ; chaque processus en crée autant qu'il en a besoin.

pub mod chat_feed;
mod chat_lines;
pub mod envelope;
pub mod events_feed;
pub mod file_secrets;
mod host;
mod lifecycle;
pub mod notifier;
pub mod original_html;
pub mod platform;
pub mod polling;
mod presence;
mod streams;

pub use host::Host;
pub use presence::Presence;
