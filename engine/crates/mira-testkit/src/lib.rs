//! Faux serveurs IMAP, SMTP et OAuth sur 127.0.0.1, utilisés uniquement par les tests.
//! Aucun crate `mira-*` n'est utilisé : le testkit voit le moteur en boîte noire.

pub mod credentials;
pub mod imap;
pub mod oauth;
pub mod smtp;

pub use credentials::Credentials;
