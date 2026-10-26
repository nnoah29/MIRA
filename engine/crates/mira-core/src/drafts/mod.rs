//! Brouillons : création (neuf, réponse, transfert), édition, pièces jointes, envoi planifié.

mod attachments;
mod compose;
mod create;
mod delete;
mod edit;
pub(crate) mod identity;
mod scheduling;
mod server_copy;
pub(crate) mod state;
mod threading;

pub use attachments::{MAX_ATTACHMENT_BYTES, attach_file, remove_attachment};
pub use create::{
    RuleReplyText, create_blank, create_forward, create_reply, create_reply_all, create_rule_reply,
};
pub use delete::delete_draft;
pub use edit::{DraftEdit, edit_draft};
pub use scheduling::{cancel_send, schedule_send};
pub use server_copy::queue_draft_copy;
pub use threading::{ThreadingHeaders, draft_threading_headers, threading_headers};
