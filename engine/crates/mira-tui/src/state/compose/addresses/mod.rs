//! Adresses d'un champ : lecture du texte saisi et écriture des adresses du moteur.

pub mod format;
pub mod parse;
pub mod segments;

pub use format::format_addresses;
pub use parse::parse_addresses;
