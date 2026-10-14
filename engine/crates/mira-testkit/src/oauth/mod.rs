//! Faux serveur d'autorisation OAuth2 (RFC 6749, PKCE RFC 7636) sur 127.0.0.1.

mod authorize;
mod browser;
mod client_secret;
mod context;
mod control;
mod error;
pub mod form;
mod http;
mod listener;
pub mod pkce;
mod redirect;
mod router;
mod server;
mod state;
mod token;

pub use server::FakeOAuthServer;
