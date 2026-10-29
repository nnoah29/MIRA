//! Masquage des données sensibles avant l'envoi à un service d'IA, et restitution dans la réponse.
//! Les détecteurs sont purs ; la table des valeurs ne vit qu'en mémoire, le temps d'une génération.

mod apply;
mod card;
mod contact;
mod debug;
mod detect;
mod iban;
mod otp;
mod placeholder;
mod restore;
mod scan;
mod secret;

pub use apply::Masker;
pub use detect::{Found, detect};
pub use restore::Restorer;
