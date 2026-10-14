//! Coffre d'identifiants au-dessus du trousseau : mots de passe, jetons et expiration (7.7).

mod credential_vault;
mod delete;
mod expiry;
mod load;
mod refresh;
mod refresh_lock;
mod store;

pub use credential_vault::CredentialVault;
pub use load::LoginSecret;
