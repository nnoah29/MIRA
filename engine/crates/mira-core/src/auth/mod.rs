//! Authentification : OAuth2 PKCE avec redirection en boucle locale, jetons et coffre d'identifiants.

pub mod flow;
pub mod form;
pub mod loopback;
pub mod pkce;

pub use flow::{AuthorizationCode, FlowPhase, PendingFlow, start_flow, wait_for_redirect};
