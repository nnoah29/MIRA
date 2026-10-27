//! Forme portable vers libellé : lecture stricte, champs inconnus et couleurs mal formées refusés.

use serde_json::Value;

use super::color::parse_color;

/// Libellé lu dans le fichier ; le nom n'est pas encore contrôlé, la couleur est facultative
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortableLabel {
    pub name: String,
    pub color: Option<u32>,
}

/// Raison pour laquelle une entrée du fichier n'est pas un libellé lisible
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortableLabelError {
    InvalidColor,
    Invalid,
}

/// Lit une entrée du fichier : objet avec `name` (texte) et `color` (texte) facultative, rien d'autre
pub fn from_portable(value: Value) -> Result<PortableLabel, PortableLabelError> {
    let map = value.as_object().ok_or(PortableLabelError::Invalid)?;
    if map.keys().any(|key| key != "name" && key != "color") {
        return Err(PortableLabelError::Invalid);
    }
    let name = map
        .get("name")
        .and_then(Value::as_str)
        .ok_or(PortableLabelError::Invalid)?
        .to_string();
    let color = map
        .get("color")
        .map(|c| {
            c.as_str()
                .and_then(parse_color)
                .ok_or(PortableLabelError::InvalidColor)
        })
        .transpose()?;
    Ok(PortableLabel { name, color })
}
