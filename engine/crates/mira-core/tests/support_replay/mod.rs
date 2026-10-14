#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Banc de test du rejeu : monde local, faux serveurs IMAP et SMTP, identifiants simulés, session connectée.

pub mod credentials;
pub mod credentials_port;
pub mod events_view;
pub mod fixture;
pub mod observe;
pub mod presets;
pub mod queue_view;
pub mod sends;
