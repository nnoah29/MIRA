//! Données brutes du serveur, et leur traduction en notions du domaine.

use serde::{Deserialize, Serialize};

/// Boîte telle que le serveur la décrit (réponse LIST).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RawMailbox {
    pub name: String,
    pub delimiter: Option<String>,
    pub attributes: Vec<String>,
}

/// Attributs FETCH propres au service, sous forme de couples (nom en majuscules, valeur brute).
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RawMessageAttributes {
    pub items: Vec<(String, String)>,
}

/// Ce que le provider déduit des attributs bruts d'un message.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ServerMessageTranslation {
    pub stable_id: Option<String>,
    pub labels: Vec<String>,
    pub thread_hint: Option<String>,
}
