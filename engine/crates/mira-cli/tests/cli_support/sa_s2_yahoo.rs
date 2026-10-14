//! Décor du scénario S2 : compte Yahoo de test, mot de passe d'application sur l'entrée standard.

use mira_testkit::imap::CapabilitySet;

use super::sa_world::SaWorld;

pub const YAHOO_USER: &str = "alice@yahoo.com";
pub const APP_PASSWORD: &str = "yahoo-APP-pass-7731";
pub const ADD_APP_PASSWORD: [&str; 8] = [
    "account",
    "add",
    "--address",
    YAHOO_USER,
    "--auth",
    "app-password",
    "--password-stdin",
    "--json",
];

/// Démarre les faux serveurs pour le compte Yahoo de test.
pub async fn yahoo_world() -> SaWorld {
    SaWorld::start(YAHOO_USER, APP_PASSWORD, CapabilitySet::full()).await
}
