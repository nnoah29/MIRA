//! Flux OAuth2 « code d'autorisation + PKCE » avec redirection en boucle locale (15.2).

mod authorize;
mod callback;
mod callback_outcome;
mod client_secret;
mod complete;
mod connection;
mod key_complete;
mod pages;
mod pending;
mod pending_debug;
mod phase;
mod redirect;
mod session_complete;
mod session_start;
mod start;
mod start_key;
mod state_check;

pub use authorize::authorize_until_cancelled;
pub use callback::AuthorizationCode;
pub use complete::complete_flow;
pub use key_complete::complete_key_flow;
pub use pending::PendingFlow;
pub use phase::FlowPhase;
pub use redirect::wait_for_redirect;
pub use session_complete::complete_session_flow;
pub use session_start::start_session_flow;
pub use start::start_flow;
pub use start_key::start_key_flow;
