//! Liste des types de contenu autorisés d'une connexion : sans doublon, dans l'ordre de première apparition.

use mira_vocabulary::ContentKind;

/// Ajoute les types donnés à la liste existante, sans doublon ni perte.
pub(super) fn merged_kinds(existing: &[ContentKind], added: &[ContentKind]) -> Vec<ContentKind> {
    distinct_kinds(&[existing, added].concat())
}

/// Garde chaque type une seule fois, dans l'ordre où il apparaît.
pub(super) fn distinct_kinds(kinds: &[ContentKind]) -> Vec<ContentKind> {
    let mut distinct = Vec::with_capacity(kinds.len());
    for kind in kinds {
        if !distinct.contains(kind) {
            distinct.push(*kind);
        }
    }
    distinct
}
