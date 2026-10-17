//! Bibliothèque du client plein écran `mira-tui` : état, vues et boucle d'événements.
//! Le binaire se limite à appeler la boucle ; tout le reste est testable sans terminal.

pub mod app;
pub mod commands;
pub mod input;
pub mod state;
pub mod theme;
pub mod update;
pub mod view;
