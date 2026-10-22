//! Règles « condition → action » : évaluateur pur (sans I/O) et, dans `apply`, exécution à l'arrivée des messages.

pub mod action_translation;
pub mod address_eval;
pub mod apply;
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
