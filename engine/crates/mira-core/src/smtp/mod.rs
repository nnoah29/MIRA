//! Envoi SMTP : composition du message MIME et client qui encapsule `lettre` (annexe H).

mod check;
pub mod compose;
mod errors;
mod login;
mod outgoing;
mod reply_codes;
mod send;
#[cfg(test)]
mod tests;
mod transport;

pub use check::check_smtp_login;
pub use login::{SmtpCredential, SmtpLogin};
pub use outgoing::OutgoingMessage;
pub use send::{send_message, send_within};
