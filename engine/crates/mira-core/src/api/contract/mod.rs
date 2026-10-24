//! Forme sérialisable du contrat : requêtes par domaine et réponses.

mod account_requests;
mod auth_requests;
mod credential_input;
mod identity_requests;
mod label_settings_requests;
mod label_settings_responses;
mod local_label_requests;
mod local_label_responses;
mod mailbox_requests;
mod maintenance_requests;
mod message_requests;
mod read_responses;
mod request;
mod response;
mod rule_requests;
mod rule_responses;
mod search_requests;
mod secret_input;
mod settings_requests;
mod settings_responses;
mod sync_requests;
mod sync_response;
mod thread_requests;
mod write;

pub use account_requests::{AccountChanges, AccountDraft, AccountRequest};
pub use auth_requests::AuthRequest;
pub use credential_input::CredentialInput;
pub use identity_requests::{IdentityRequest, NewIdentity};
pub use label_settings_requests::LabelSettingsRequest;
pub use label_settings_responses::{LabelOverview, LabelSettingsResponse};
pub use local_label_requests::{ImportedLabel, LocalLabelRequest};
pub use local_label_responses::{ImportReport, LocalLabelResponse};
pub use mailbox_requests::MailboxRequest;
pub use maintenance_requests::MaintenanceRequest;
pub use message_requests::{AttachmentTarget, MessageRequest, RemoteContentTarget};
pub use read_responses::{ExportReport, IntegrityReport, RawSourceView, ThreadView};
pub use request::{EngineRequest, EventRequest, OperationRequest, ProviderRequest, Request};
pub use response::{
    AccountPreparation, AccountStatus, ContractInfo, EngineStatus, OAuthStart, Response,
};
pub use rule_requests::RuleRequest;
pub use rule_responses::{CatalogEntry, RuleCatalog, RulePreview, RuleResponse, RuleView};
pub use search_requests::SearchRequest;
pub use secret_input::SecretInput;
pub use settings_requests::SettingsRequest;
pub use settings_responses::SettingsResponse;
pub use sync_requests::{SyncRequest, SyncScope};
pub use sync_response::{AccountSyncStatus, SyncResponse};
pub use thread_requests::ThreadRequest;
pub use write::{
    ActionRequest, ComposeRequest, DraftChanges, DraftMode, OutboxEntry, QueueRequest,
    TargetSelection, WriteResponse,
};
