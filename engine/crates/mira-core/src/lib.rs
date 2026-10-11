//! Cœur du moteur : ne nomme aucun provider (D-06), raisonne en capacités.
//! Chaque sous-domaine est un module (D-34) ; chaque bibliothèque externe reste dans son module (annexe H).

pub mod capabilities;
pub mod events;
pub mod mailboxes;
pub mod ports;
pub mod profile;
pub mod reading;
pub mod registry;
pub mod storage;
pub mod threads;
