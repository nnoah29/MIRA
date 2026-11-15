//! Style de rédaction : analyse locale des mails envoyés (fonctions pures) et consigne de style pour la rédaction.

mod analyze;
mod clean;
mod greetings;
mod measures;
mod register;
mod render;

pub use analyze::analyze;
pub use clean::clean_mail;
pub use render::render_style_notice;
