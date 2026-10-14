//! Synchronisation descendante (ch. 10) : descente par étapes, stratégies, identité, plage, temps réel.

mod context;
mod download;
mod identity;
mod realtime;
mod resync;
mod session;
mod strategy;
mod supervisor;
mod window;

pub use context::SyncContext;
pub use download::{sync_account, sync_inbox};
pub use realtime::BackoffPolicy;
pub use session::{
    BoxFuture, Connection, ConnectionBudget, CredentialProvider, SyncSession, open_connection,
};
pub use supervisor::{SupervisorConfig, SyncSupervisor};
pub use window::{ExtendOutcome, extend_window, start_extend_window};
