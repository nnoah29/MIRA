//! Forme sérialisable du contrat : requêtes par domaine et réponses.

mod account_requests;
mod auth_requests;
mod credential_input;
mod identity_requests;
mod mailbox_requests;
mod maintenance_requests;
mod message_requests;
mod read_responses;
mod request;
mod response;
mod search_requests;
mod secret_input;
mod sync_requests;
mod sync_response;
mod thread_requests;
mod write;

pub use account_requests::{AccountChanges, AccountDraft, AccountRequest};
pub use auth_requests::AuthRequest;
pub use credential_input::CredentialInput;
pub use identity_requests::{IdentityRequest, NewIdentity};
pub use mailbox_requests::MailboxRequest;
pub use maintenance_requests::MaintenanceRequest;
pub use message_requests::{AttachmentTarget, MessageRequest, RemoteContentTarget};
pub use read_responses::{ExportReport, IntegrityReport, RawSourceView, ThreadView};
pub use request::{EngineRequest, EventRequest, OperationRequest, ProviderRequest, Request};
pub use response::{
    AccountPreparation, AccountStatus, ContractInfo, EngineStatus, OAuthStart, Response,
};
pub use search_requests::SearchRequest;
pub use secret_input::SecretInput;
pub use sync_requests::{SyncRequest, SyncScope};
pub use sync_response::{AccountSyncStatus, SyncResponse};
pub use thread_requests::ThreadRequest;
pub use write::{
    ActionRequest, ComposeRequest, DraftChanges, DraftMode, OutboxEntry, QueueRequest,
    TargetSelection, WriteResponse,
};
