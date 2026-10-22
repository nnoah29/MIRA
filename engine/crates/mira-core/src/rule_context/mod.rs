//! Passerelle entre le stockage et l'évaluateur pur des règles : monde (comptes, boîtes) et contexte d'un message.

mod message;
mod world;

pub use message::{message_context, own_addresses};
pub use world::StoredRuleWorld;
