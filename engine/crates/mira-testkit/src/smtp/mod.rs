//! Faux serveur SMTP (RFC 5321) avec pannes injectables.

mod commands;
mod control;
mod listener;
mod mail;
mod server;
mod session;
mod state;

pub use mail::ReceivedMail;
pub use server::FakeSmtpServer;
