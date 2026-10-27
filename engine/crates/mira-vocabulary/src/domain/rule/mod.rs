//! Règles « condition → action » appliquées aux nouveaux messages.

pub mod action;
pub mod condition;
pub mod import_report;
pub mod matcher;
#[allow(clippy::module_inception)]
pub mod rule;
pub mod test;
