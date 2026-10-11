//! Extensions IMAP activables une à une pour le faux serveur.

mod capability;
mod names;
mod presets;
mod set;

pub use capability::Capability;
pub use set::CapabilitySet;
