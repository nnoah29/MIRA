//! Vue « boîte d'envoi et file d'actions » : ouverture, lectures, réessayer, annuler, envois incertains.

pub mod change;
pub mod events;
pub mod intent;
pub mod open;
pub mod popup;
pub mod route;

pub use intent::QueueIntent;
