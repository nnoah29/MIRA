//! Bibliothèque du client plein écran `mira-tui` : état, vues et boucle d'événements.
//! Le binaire se limite à appeler la boucle ; tout le reste est testable sans terminal.

pub mod app;
pub mod commands;
pub mod state;
pub mod update;
