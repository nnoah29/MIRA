//! Fabriques d'états, de commandes et de réponses pour les tests de l'éditeur de brouillon.
#![allow(dead_code, unused_imports)]

mod commands;
mod data;
mod render;
mod typing;

pub use commands::{
    create_command, created_reply, delete_command, deleted_reply, saved_reply, update_command,
};
pub use data::{draft, open_modified, open_reply, open_with};
pub use render::{screen, text_of};
pub use typing::{compose, edit, focus_on, subject_only, type_in_draft};
