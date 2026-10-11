//! Envoi SMTP : composition du message MIME et client qui encapsule `lettre` (annexe H).

pub mod compose;
mod outgoing;

pub use outgoing::OutgoingMessage;
