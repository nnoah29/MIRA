//! Envoi SMTP : composition du message MIME et client qui encapsule `lettre` (annexe H).

pub mod compose;
mod errors;
mod login;
mod outgoing;
mod reply_codes;
mod send;
#[cfg(test)]
mod tests;
mod transport;

pub use login::{SmtpCredential, SmtpLogin};
pub use outgoing::OutgoingMessage;
pub use send::{send_message, send_within};
