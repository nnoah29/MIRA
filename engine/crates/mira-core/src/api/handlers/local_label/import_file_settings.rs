//! Réglages d'un libellé importé : écrits pour un libellé neuf, fusionnés pour un libellé remplacé.

use mira_vocabulary::{LabelKind, LabelRef, LabelSettings, LocalLabelId, MiraError};

use crate::portability::labels::PortableSettings;
use crate::storage::Tx;

/// Désignation du libellé local pour la table des réglages
fn local_ref(id: &LocalLabelId) -> LabelRef {
    LabelRef {
        kind: LabelKind::Local,
        id: id.as_str().to_string(),
    }
}

/// Réglages d'un libellé neuf : ceux du fichier sur les défauts, rien d'écrit si le fichier n'en a pas
pub(super) fn write_new(
    tx: &Tx<'_>,
    id: &LocalLabelId,
    settings: &PortableSettings,
) -> Result<(), MiraError> {
    if settings.is_empty() {
        return Ok(());
    }
    let merged = settings.applied_to(LabelSettings::default());
    tx.put_label_settings(&local_ref(id), &merged)
}

/// Réglages d'un libellé présent alignés sur le fichier ; vrai si quelque chose a changé
pub(super) fn merge_existing(
    tx: &Tx<'_>,
    id: &LocalLabelId,
    settings: &PortableSettings,
) -> Result<bool, MiraError> {
    let reference = local_ref(id);
    let before = tx.get_label_settings(&reference)?;
    let after = settings.applied_to(before.clone());
    if after == before {
        return Ok(false);
    }
    tx.put_label_settings(&reference, &after)?;
    Ok(true)
}
