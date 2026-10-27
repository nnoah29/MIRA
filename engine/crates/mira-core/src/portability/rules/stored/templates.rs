//! Modèles de message retrouvés par nom, casse et espaces de bord ignorés.

use mira_vocabulary::TemplateId;

use super::StoredRefs;
use crate::portability::rules::TemplateRefs;

impl TemplateRefs for StoredRefs<'_, '_> {
    /// Nom du modèle
    fn template_name(&self, id: &TemplateId) -> Option<String> {
        let template = self.absorb(self.tx.get_template(id))?;
        Some(template.name)
    }

    /// Modèle portant ce nom
    fn template_by_name(&self, name: &str) -> Option<TemplateId> {
        let wanted = name.trim().to_lowercase();
        let all = self.absorb(self.tx.list_templates().map(Some))?;
        all.into_iter()
            .find(|template| template.name.trim().to_lowercase() == wanted)
            .map(|template| template.id)
    }
}
