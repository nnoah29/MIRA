//! Filtres texte, champs d'adresse et comparaisons numériques des conditions.

use serde::{Deserialize, Serialize};

/// Opérateur de comparaison de texte, casse ignorée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextOp {
    Contains,
    Equals,
    StartsWith,
    EndsWith,
    Regex,
    InList,
}

/// Filtre texte : un opérateur et une ou plusieurs valeurs.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TextMatcher {
    pub op: TextOp,
    pub values: Vec<String>,
}

/// Partie d'une adresse sur laquelle porte un filtre.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AddressField {
    Address,
    Name,
    Domain,
}

/// Rôle d'un destinataire.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipientKind {
    To,
    Cc,
    Bcc,
}

/// Sens d'une comparaison numérique.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Comparison {
    Less,
    Equal,
    Greater,
}
