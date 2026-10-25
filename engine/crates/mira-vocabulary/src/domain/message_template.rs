//! Modèle de message : texte réutilisable avec variables, propre à l'app et jamais envoyé au serveur.

use serde::{Deserialize, Serialize};

use crate::ids::kinds::TemplateId;

/// Modèle de message : nom unique (sans tenir compte de la casse), objet optionnel et corps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageTemplate {
    pub id: TemplateId,
    pub name: String,
    pub subject: Option<String>,
    pub body: String,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Destinataire principal du message en cours de rédaction.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContactInfo {
    pub name: Option<String>,
    pub first_name: Option<String>,
    pub email: Option<String>,
}

/// Valeurs fournies par l'app pour remplacer les variables ; une valeur absente ou vide laisse la variable visible.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderContext {
    pub contact: Option<ContactInfo>,
    pub account_name: Option<String>,
    pub account_email: Option<String>,
    pub subject: Option<String>,
    pub date: Option<String>,
    pub time: Option<String>,
}

/// Modèle rendu : objet et corps remplacés, variables restées visibles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderedTemplate {
    pub subject: Option<String>,
    pub body: String,
    pub unresolved: Vec<String>,
}
