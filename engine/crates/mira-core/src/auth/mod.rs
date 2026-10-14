//! Authentification : OAuth2 PKCE avec redirection en boucle locale, jetons et coffre d'identifiants.

pub mod client_config;
pub mod flow;
pub mod flow_table;
pub mod form;
pub mod loopback;
pub mod pkce;
pub(crate) mod renewal;
pub mod tokens;
pub mod vault;

pub use client_config::OAuthClientConfig;
pub use flow::{
    AuthorizationCode, FlowPhase, PendingFlow, authorize_until_cancelled, complete_flow,
    start_flow, wait_for_redirect,
};
pub use flow_table::{FlowEntry, FlowSummary, FlowTarget, OAuthFlows};
pub use tokens::{TokenSet, exchange_code, refresh_tokens};
pub use vault::{CredentialVault, LoginSecret};
