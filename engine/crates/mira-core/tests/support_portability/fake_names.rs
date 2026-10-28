#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, dead_code)]
//! Libellé `l1` (`Compta`) et modèle `t1` (`Accusé de réception`) factices.

use mira_core::portability::rules::{LabelRefs, TemplateRefs};
use mira_vocabulary::{LocalLabelId, TemplateId};

use super::fake_refs::FakeRefs;

/// Nom normalisé : espaces de bord retirés, minuscules
fn key(name: &str) -> String {
    name.trim().to_lowercase()
}

impl LabelRefs for FakeRefs {
    /// Nom du libellé `l1`, sauf s'il a disparu
    fn label_name(&self, id: &LocalLabelId) -> Option<String> {
        (id.as_str() == "l1" && !self.without_label).then(|| "Compta".to_string())
    }
    /// Libellé `l1` pour son nom
    fn label_by_name(&self, name: &str) -> Option<LocalLabelId> {
        (key(name) == "compta" && !self.without_label).then(|| LocalLabelId::from_raw("l1"))
    }
}

impl TemplateRefs for FakeRefs {
    /// Nom du modèle `t1`
    fn template_name(&self, id: &TemplateId) -> Option<String> {
        (id.as_str() == "t1").then(|| "Accusé de réception".to_string())
    }
    /// Modèle `t1` pour son nom
    fn template_by_name(&self, name: &str) -> Option<TemplateId> {
        (key(name) == "accusé de réception").then(|| TemplateId::from_raw("t1"))
    }
}
