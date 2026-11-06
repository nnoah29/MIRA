#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Jeu de contrat commun : les mêmes scénarios, exécutés par chaque chemin qui mène au moteur.

pub mod driver;
pub mod errors;
pub mod mail_events;
pub mod mail_fixture;
pub mod mail_html;
pub mod mail_samples;
pub mod mail_start;
pub mod mail_values;
pub mod suite;
