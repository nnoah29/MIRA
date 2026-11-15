//! Cartes de messages que l'IA montre dans le chat : un identifiant du moteur et, si elle en a une, la raison.

use serde::{Deserialize, Serialize};

/// Un message à montrer : l'identifiant donné par les outils de lecture, et pourquoi il compte.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MailCard {
    pub id: String,
    pub reason: Option<String>,
}

/// Lot de cartes d'un appel de l'outil `show_mails`, avec un titre facultatif ; l'application vérifie chaque identifiant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MailCards {
    pub title: Option<String>,
    pub cards: Vec<MailCard>,
}
