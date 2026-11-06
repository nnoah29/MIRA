#![allow(dead_code, clippy::unwrap_used, clippy::expect_used, clippy::panic)]
//! Jeu de contrat commun : les mêmes scénarios, exécutés par chaque chemin qui mène au moteur.

pub mod chat_cancel;
pub mod chat_draft;
pub mod chat_errors;
#[allow(clippy::duplicate_mod)]
pub mod chat_fixture;
pub mod chat_lines;
pub mod chat_masked;
pub mod chat_start;
pub mod chat_tools;
pub mod driver;
pub mod errors;
pub mod labels;
pub mod mail_events;
pub mod mail_fixture;
pub mod mail_html;
pub mod mail_samples;
pub mod mail_start;
pub mod mail_values;
pub mod rules;
pub mod suite_chat;
pub mod suite_chat_errors;
pub mod suite_chat_extras;
pub mod suite_data;
pub mod suite_mail;
