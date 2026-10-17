//! Réponse complète à une commande : lignes non étiquetées et statut final.

use crate::imap::parse::status::ResponseCode;

/// Résultat final d'une commande.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StatusKind {
    Ok,
    No,
    Bad,
}

/// Ligne de statut étiquetée : résultat, code de réponse éventuel, texte du serveur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Status {
    pub kind: StatusKind,
    pub code: Option<ResponseCode>,
    pub text: String,
}

/// Réponse à une commande ; les lignes non étiquetées n'ont plus leur préfixe `* `.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Response {
    pub untagged: Vec<Vec<u8>>,
    pub status: Status,
}
