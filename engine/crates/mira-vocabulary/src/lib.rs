//! Vocabulaire commun : types du domaine et interface provider (D-33).

pub mod address;
pub mod contract;
pub mod error;
pub mod ids;
pub mod time;

pub use address::EmailAddress;
pub use address::entry::AddressEntry;
pub use contract::version::{CONTRACT_VERSION, ContractVersion};
pub use error::action::SuggestedAction;
pub use error::category::ErrorCategory;
pub use error::code::ErrorCode;
pub use error::context::ErrorContext;
pub use error::mira_error::MiraError;
pub use ids::Id;
pub use ids::keys::{Cursor, EventSeq, IdempotencyKey, ProviderId};
pub use ids::kinds::{
    AccountId, ActionId, AttachmentId, DraftId, IdentityId, MailboxId, MessageId, OperationId,
    ProfileId, ThreadId, ViewId,
};
pub use time::Timestamp;
