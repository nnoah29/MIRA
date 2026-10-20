//! Rendu HTML sûr (11.6, 15.4) : seul module qui connaît `ammonia`.
//! Le moteur livre à tous ses clients un HTML sans script, ressources distantes bloquées par défaut.

mod limits;
mod remote;
mod sanitize;

pub use sanitize::{SafeHtml, sanitize_html};
