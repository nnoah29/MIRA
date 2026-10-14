//! Contrat d'API du moteur (ch. 12, D-26, AG-04) : `Engine::execute(Request) -> Response`.
//! Chaque opération a son propre handler, atteint par un routage `match` exhaustif par domaine.

mod config;
pub mod contract;
mod engine;
mod handlers;
mod queue_access;
mod router;
mod startup;

pub use config::EngineConfig;
pub use contract::{
    AccountChanges, AccountDraft, AccountRequest, AccountStatus, ActionRequest, AuthRequest,
    ComposeRequest, ContractInfo, CredentialInput, DraftChanges, DraftMode, EngineRequest,
    EngineStatus, EventRequest, IdentityRequest, NewIdentity, OperationRequest, OutboxEntry,
    ProviderRequest, QueueRequest, Request, Response, SecretInput, TargetSelection, WriteResponse,
};
pub use engine::Engine;
