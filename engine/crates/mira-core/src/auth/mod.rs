//! Authentification : OAuth2 PKCE avec redirection en boucle locale, jetons et coffre d'identifiants.

pub mod client_config;
pub mod flow;
pub mod form;
pub mod loopback;
pub mod pkce;
pub mod tokens;
pub mod vault;

pub use client_config::OAuthClientConfig;
pub use flow::{
    AuthorizationCode, FlowPhase, PendingFlow, complete_flow, start_flow, wait_for_redirect,
};
pub use tokens::{TokenSet, exchange_code, refresh_tokens};
pub use vault::{CredentialVault, LoginSecret};
