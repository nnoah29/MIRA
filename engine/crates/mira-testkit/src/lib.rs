//! Faux serveurs IMAP, SMTP, OAuth et IA sur 127.0.0.1, utilisés uniquement par les tests.
//! Aucun crate `mira-*` n'est utilisé : le testkit voit le moteur en boîte noire.

pub mod ai;
pub mod credentials;
pub mod imap;
pub mod oauth;
pub mod smtp;

pub use credentials::Credentials;
