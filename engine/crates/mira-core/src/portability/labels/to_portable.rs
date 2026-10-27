//! Libellé vers sa forme portable : nom et couleur lisible, sans identifiant.

use mira_vocabulary::LocalLabel;
use serde_json::{Value, json};

use super::color::format_color;

/// Forme portable d'un libellé local
pub fn to_portable(label: &LocalLabel) -> Value {
    json!({ "name": label.name, "color": format_color(label.color) })
}
