//! Vocabulaire commun : types du domaine et interface provider (D-33).

pub mod address;
pub mod contract;
pub mod domain;
pub mod error;
pub mod ids;
pub mod time;

pub use address::EmailAddress;
pub use address::entry::AddressEntry;
pub use contract::version::{CONTRACT_VERSION, ContractVersion};
pub use domain::account::{Account, AccountState, SyncSettings};
pub use domain::action::{ActionKind, ActionState, PendingAction};
pub use domain::appearance::Appearance;
pub use domain::attachment::{Attachment, CacheState};
pub use domain::auth::{AuthMethod, AuthMethodKind, OAuthSettings, SecretKind, SecretRef};
pub use domain::draft::{Draft, DraftAttachment, DraftState};
pub use domain::identity::Identity;
pub use domain::mailbox::{Mailbox, MailboxRole, SyncMarker};
pub use domain::message::{ContentState, Flags, MessageSummary, Unsubscribe};
pub use domain::page::{Page, PageRequest};
pub use domain::servers::{Security, ServerEndpoint, ServerSettings};
pub use domain::thread::Thread;
pub use domain::view::{StandardView, ViewRef};
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
