//! Vue « Brouillons » : ouverture, lectures, réouverture d'un brouillon enregistré, suppression en deux temps.

pub mod delete;
pub mod events;
pub mod intent;
pub mod open;
pub mod reopen;
pub mod reply;
pub mod route;
pub mod target;

pub use intent::DraftsIntent;
