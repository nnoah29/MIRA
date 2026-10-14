//! Forme sérialisable du contrat : requêtes par domaine et réponses.

mod account_requests;
mod auth_requests;
mod credential_input;
mod identity_requests;
mod request;
mod response;
mod secret_input;
mod write;

pub use account_requests::{AccountChanges, AccountDraft, AccountRequest};
pub use auth_requests::AuthRequest;
pub use credential_input::CredentialInput;
pub use identity_requests::{IdentityRequest, NewIdentity};
pub use request::{EngineRequest, EventRequest, OperationRequest, ProviderRequest, Request};
pub use response::{
    AccountPreparation, AccountStatus, ContractInfo, EngineStatus, OAuthStart, Response,
};
pub use secret_input::SecretInput;
pub use write::{
    ActionRequest, ComposeRequest, DraftChanges, DraftMode, OutboxEntry, QueueRequest,
    TargetSelection, WriteResponse,
};
