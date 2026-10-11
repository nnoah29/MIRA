//! Vocabulaire commun : types du domaine et interface provider (D-33).

pub mod ids;
pub mod time;

pub use ids::Id;
pub use ids::keys::{Cursor, EventSeq, IdempotencyKey, ProviderId};
pub use ids::kinds::{
    AccountId, ActionId, AttachmentId, DraftId, IdentityId, MailboxId, MessageId, OperationId,
    ProfileId, ThreadId, ViewId,
};
pub use time::Timestamp;
