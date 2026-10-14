//! Session de synchronisation : budget de connexions, identifiants, connexion et opération en cours.

mod budget;
mod budgets;
mod connection;
mod credentials;
mod lease;
mod sync_session;
mod waiting;

pub use budget::ConnectionBudget;
pub use budgets::ConnectionBudgets;
pub use connection::{Connection, open_connection};
pub use credentials::{BoxFuture, CredentialProvider};
pub use lease::LeasedClient;
pub use sync_session::SyncSession;
