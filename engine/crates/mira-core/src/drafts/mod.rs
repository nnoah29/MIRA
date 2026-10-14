//! Brouillons : création (neuf, réponse, transfert), édition, pièces jointes, envoi planifié.

mod attachments;
mod compose;
mod create;
mod delete;
mod edit;
pub(crate) mod identity;
mod scheduling;
pub(crate) mod state;
mod threading;

pub use attachments::{MAX_ATTACHMENT_BYTES, attach_file, remove_attachment};
pub use create::{create_blank, create_forward, create_reply, create_reply_all};
pub use delete::delete_draft;
pub use edit::{DraftEdit, edit_draft};
pub use scheduling::{cancel_send, schedule_send};
pub use threading::{ThreadingHeaders, draft_threading_headers, threading_headers};
