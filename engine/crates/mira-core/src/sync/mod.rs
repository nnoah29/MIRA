//! Synchronisation descendante (ch. 10) : descente par étapes, stratégies, identité, plage, temps réel.

mod context;
mod download;
mod identity;
mod mode;
mod realtime;
mod report;
mod requests;
mod resync;
mod rule_hook;
mod session;
mod settings_hook;
mod strategy;
mod supervisor;
mod window;

pub use context::SyncContext;
pub use download::{sync_account, sync_inbox, sync_requested};
pub use identity::Ingested;
pub use mode::BackgroundSync;
pub use realtime::BackoffPolicy;
pub use report::{ReportHandle, SyncReport};
pub use requests::{RequestScope, SyncRequests, TakenRequests, Ticket};
pub use rule_hook::apply_rules_to_message;
pub use session::{
    BoxFuture, Connection, ConnectionBudget, ConnectionBudgets, CredentialProvider, LeasedClient,
    SyncSession, open_connection,
};
pub use strategy::{Announce, store_batch};
pub use supervisor::{SupervisorConfig, SyncSupervisor};
pub use window::{ExtendOutcome, extend_window, start_extend_window};
