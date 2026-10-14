//! Envoi d'un brouillon : MIME, SMTP, copie dans Envoyés, reprise d'un envoi interrompu (10.8, décision 9).

mod deliver;
mod draft_copy;
mod execute;
mod message_id;
mod prepare;
mod resolve;
mod role_mailbox;
mod sent_copy;
mod settle;

pub use resolve::resolve_uncertain_send;

pub(crate) use draft_copy::save_draft_copy;
pub(crate) use execute::execute_send;
pub(crate) use message_id::prepare_send_attempt;
pub(crate) use settle::{recover_interrupted_send, settle_interrupted_send};
