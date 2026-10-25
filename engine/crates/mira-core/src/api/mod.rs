//! Contrat d'API du moteur (ch. 12, D-26, AG-04) : `Engine::execute(Request) -> Response`.
//! Chaque opération a son propre handler, atteint par un routage `match` exhaustif par domaine.

mod config;
pub mod contract;
mod engine;
mod handlers;
mod queue_access;
mod router;
mod services;
mod startup;

pub use config::EngineConfig;
pub use contract::{
    AccountChanges, AccountDraft, AccountRequest, AccountStatus, AccountSyncStatus, ActionRequest,
    AttachmentTarget, AuthRequest, CatalogEntry, ComposeRequest, ContractInfo, CredentialInput,
    DraftChanges, DraftMode, EngineRequest, EngineStatus, EventRequest, IdentityRequest,
    ImportReport, ImportedLabel, LabelOverview, LabelSettingsRequest, LabelSettingsResponse,
    LocalLabelRequest, LocalLabelResponse, MailboxRequest, MaintenanceRequest, MessageRequest,
    NewIdentity, OperationRequest, OutboxEntry, ProviderRequest, QueueRequest, RemoteContentTarget,
    Request, Response, RuleCatalog, RulePreview, RuleRequest, RuleResponse, RuleView,
    SearchRequest, SecretInput, SettingsRequest, SettingsResponse, SyncRequest, SyncResponse,
    SyncScope, TargetSelection, ThreadRequest, WriteResponse,
};
pub use engine::Engine;
