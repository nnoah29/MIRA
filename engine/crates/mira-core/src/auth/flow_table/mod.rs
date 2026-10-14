//! Flux OAuth en cours dans le moteur : enregistrement, prise en charge, annulation.

mod entry;
mod errors;
mod lifecycle;
mod table;

pub use entry::{FlowEntry, FlowSummary, FlowTarget};
pub use table::OAuthFlows;
