//! Cœur du moteur : ne nomme aucun provider (D-06), raisonne en capacités.
//! Chaque sous-domaine est un module (D-34) ; chaque bibliothèque externe reste dans son module (annexe H).

pub mod accounts;
pub mod actions;
pub mod api;
pub mod auth;
pub mod capabilities;
pub mod drafts;
pub mod events;
pub mod html;
pub mod imap;
pub mod mailboxes;
pub mod mime;
pub mod operations;
pub mod ports;
pub mod profile;
pub mod reading;
pub mod registry;
pub mod send;
pub mod smtp;
pub mod storage;
pub mod sync;
pub mod threads;
