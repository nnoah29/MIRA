//! Session de synchronisation : budget de connexions, identifiants, connexion et opération en cours.

mod budget;
mod connection;
mod credentials;
mod sync_session;

pub use budget::ConnectionBudget;
pub use connection::{Connection, open_connection};
pub use credentials::{BoxFuture, CredentialProvider};
pub use sync_session::SyncSession;
