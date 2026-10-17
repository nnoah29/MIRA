//! Flux OAuth2 « code d'autorisation + PKCE » avec redirection en boucle locale (15.2).

mod authorize;
mod callback;
mod client_secret;
mod complete;
mod connection;
mod pending;
mod pending_debug;
mod phase;
mod redirect;
mod start;
mod state_check;

pub use authorize::authorize_until_cancelled;
pub use callback::AuthorizationCode;
pub use complete::complete_flow;
pub use pending::PendingFlow;
pub use phase::FlowPhase;
pub use redirect::wait_for_redirect;
pub use start::start_flow;
