//! Doublures et monde de test partagés par les tests d'intégration (terminal, moteur, miroir).
#![allow(dead_code)]

pub mod config;
pub mod event_log;
pub mod failing_writer;
pub mod imap_account;
pub mod recording_raw;
pub mod recording_writer;
pub mod seed;
pub mod world;
