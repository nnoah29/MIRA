//! Code d'erreur stable, de la forme `<catégorie>.<sujet>`.

use serde::Serialize;

/// Code d'erreur stable du catalogue ; sérialisé comme une chaîne.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct ErrorCode(pub &'static str);
