//! Libellés locaux retrouvés par nom, casse et espaces de bord ignorés.

use mira_vocabulary::LocalLabelId;

use super::StoredRefs;
use crate::portability::rules::LabelRefs;

impl LabelRefs for StoredRefs<'_, '_> {
    /// Nom du libellé
    fn label_name(&self, id: &LocalLabelId) -> Option<String> {
        let label = self.absorb(self.tx.get_local_label(id))?;
        Some(label.name)
    }

    /// Libellé portant ce nom
    fn label_by_name(&self, name: &str) -> Option<LocalLabelId> {
        let label = self.absorb(self.tx.find_local_label_by_name(name))?;
        Some(label.id)
    }
}
