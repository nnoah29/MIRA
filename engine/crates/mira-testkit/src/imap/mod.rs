//! Faux serveur IMAP (RFC 9051 / 3501 et extensions activables).

pub(crate) mod capabilities;
pub(crate) mod commands;
pub(crate) mod date;
pub(crate) mod message;
pub(crate) mod parser;
pub(crate) mod response;
pub(crate) mod server;
pub(crate) mod session;
pub(crate) mod store;

pub use capabilities::{Capability, CapabilitySet};
pub use server::FakeImapServer;
pub use store::{FakeMailStore, FakeMessageView};
