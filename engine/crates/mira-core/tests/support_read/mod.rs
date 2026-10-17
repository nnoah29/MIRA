#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Monde de test des lectures par la façade : moteur, faux serveur IMAP et miroir garni à côté du moteur.
//! Les tests qui l'utilisent déclarent aussi `support`, `support_api` et `support_mirror`.

pub mod calls;
pub mod html;
pub mod mail;
pub mod operations;
pub mod requests;
pub mod seed;
pub mod server_mail;
pub mod world;
