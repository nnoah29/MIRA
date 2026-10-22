//! Réglages d'un libellé (serveur ou local) : affichage, masquage de la Réception, notifications, compteur, couleur, ordre.

use serde::{Deserialize, Serialize};

/// Nature d'un libellé : boîte « autre » d'un compte, ou libellé local.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LabelKind {
    Server,
    Local,
}

impl LabelKind {
    /// Texte stocké en base
    pub fn as_str(&self) -> &'static str {
        match self {
            LabelKind::Server => "server",
            LabelKind::Local => "local",
        }
    }
}

/// Désigne un libellé : sa nature et son identifiant (boîte ou libellé local).
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct LabelRef {
    pub kind: LabelKind,
    pub id: String,
}

/// Ce que le compteur du libellé affiche dans la barre latérale.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CounterMode {
    Unread,
    Total,
    None,
}

impl CounterMode {
    /// Texte stocké en base
    pub fn as_str(&self) -> &'static str {
        match self {
            CounterMode::Unread => "unread",
            CounterMode::Total => "total",
            CounterMode::None => "none",
        }
    }

    /// Relit le texte stocké ; une valeur inconnue donne le compteur par défaut
    pub fn from_db(text: &str) -> Self {
        match text {
            "total" => CounterMode::Total,
            "none" => CounterMode::None,
            _ => CounterMode::Unread,
        }
    }
}

/// Réglages d'un libellé ; l'absence de ligne en base vaut les réglages par défaut.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelSettings {
    pub hidden: bool,
    pub hide_from_inbox: bool,
    pub notifications: bool,
    pub counter: CounterMode,
    pub color: Option<u32>,
    pub position: Option<u32>,
}

impl Default for LabelSettings {
    /// Libellé visible, non masqué de la Réception, notifiant, compteur des non lus
    fn default() -> Self {
        Self {
            hidden: false,
            hide_from_inbox: false,
            notifications: true,
            counter: CounterMode::Unread,
            color: None,
            position: None,
        }
    }
}
