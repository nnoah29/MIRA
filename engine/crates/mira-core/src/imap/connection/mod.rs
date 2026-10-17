//! Établissement de la connexion : politique du clair, TCP, TLS, STARTTLS, authentification.

pub(crate) mod authenticate;
pub(crate) mod capabilities;
mod connect;
pub(crate) mod greeting;
pub(crate) mod open;
pub(crate) mod policy;
pub(crate) mod starttls;
pub(crate) mod tcp;
pub(crate) mod tls;
