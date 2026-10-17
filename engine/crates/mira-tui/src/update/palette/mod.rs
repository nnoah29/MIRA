//! Palette de commandes `:` : ouverture, saisie et exécution d'une commande connue.

pub mod intent;
pub mod parse;
pub mod route;

pub use intent::PaletteIntent;
pub use parse::parse_command;
