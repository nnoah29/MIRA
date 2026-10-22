//! Évaluateur pur des règles « condition → action » : aucune I/O, aucun accès à la base.

pub mod action_translation;
pub mod address_eval;
pub mod compare_eval;
pub mod condition_eval;
pub mod content_eval;
pub mod context;
pub mod date_eval;
pub mod evaluate;
pub mod group_eval;
pub mod outcome;
pub mod own_address_eval;
pub mod text_match;
pub mod validate;
