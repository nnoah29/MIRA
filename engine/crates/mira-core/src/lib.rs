//! Cœur du moteur : ne nomme aucun provider (D-06), raisonne en capacités.
//! Chaque sous-domaine est un module (D-34) ; chaque bibliothèque externe reste dans son module (annexe H).

pub mod accounts;
pub mod actions;
pub mod api;
pub mod auth;
mod body_storage;
pub mod capabilities;
pub mod drafts;
pub mod events;
pub mod fetching;
pub mod html;
pub mod imap;
pub mod labels;
pub mod mailboxes;
pub mod maintenance;
pub mod mime;
pub mod operations;
pub mod ports;
pub mod profile;
pub mod reading;
pub mod registry;
mod replayer;
pub mod rule_context;
pub mod rules;
pub mod send;
pub mod smtp;
pub mod storage;
pub mod sync;
mod sync_service;
pub mod templates;
pub mod threads;
