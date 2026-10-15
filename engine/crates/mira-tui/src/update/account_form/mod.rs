//! Formulaire d'ajout de compte par mot de passe : ouverture, saisie, préparation, envoi et réponses.

pub mod added;
pub mod draft;
pub mod edit;
pub mod focus;
pub mod intent;
pub mod open;
pub mod prepare;
pub mod route;
pub mod servers;
pub mod submit;

pub use intent::AccountFormIntent;
