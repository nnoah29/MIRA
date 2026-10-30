//! Niveau de risque des requêtes : règles, libellés, modèles et réglages.

use mira_vocabulary::ToolRisk;

use crate::api::contract::{
    LabelSettingsRequest, LocalLabelRequest, RuleRequest, SettingsRequest, SystemLabelRequest,
    TemplateRequest,
};

/// Niveau de risque des requêtes du domaine rule.
pub(super) fn rule_risk(request: &RuleRequest) -> ToolRisk {
    match request {
        RuleRequest::ListRules
        | RuleRequest::GetRule { .. }
        | RuleRequest::ValidateRule { .. }
        | RuleRequest::PreviewRule { .. }
        | RuleRequest::ListRuleCatalog
        | RuleRequest::ExportRules => ToolRisk::Read,
        RuleRequest::CreateRule { .. }
        | RuleRequest::UpdateRule { .. }
        | RuleRequest::SetRuleEnabled { .. }
        | RuleRequest::ReorderRules { .. }
        | RuleRequest::ImportRules { .. } => ToolRisk::Reversible,
        RuleRequest::DeleteRule { .. } | RuleRequest::ApplyRuleToExisting { .. } => {
            ToolRisk::Confirmed
        }
    }
}

/// Niveau de risque des requêtes du domaine local label.
pub(super) fn local_label_risk(request: &LocalLabelRequest) -> ToolRisk {
    match request {
        LocalLabelRequest::ListLocalLabels
        | LocalLabelRequest::ListLocalLabelAssignments
        | LocalLabelRequest::ExportLocalLabelsFile => ToolRisk::Read,
        LocalLabelRequest::CreateLocalLabel { .. }
        | LocalLabelRequest::RenameLocalLabel { .. }
        | LocalLabelRequest::RecolorLocalLabel { .. }
        | LocalLabelRequest::SetLocalLabel { .. }
        | LocalLabelRequest::ImportLocalLabels { .. }
        | LocalLabelRequest::ImportLocalLabelsFile { .. } => ToolRisk::Reversible,
        LocalLabelRequest::DeleteLocalLabel { .. } => ToolRisk::Confirmed,
    }
}

/// Niveau de risque des requêtes du domaine system label.
pub(super) fn system_label_risk(request: &SystemLabelRequest) -> ToolRisk {
    match request {
        SystemLabelRequest::ListSystemLabelAssignments => ToolRisk::Read,
        SystemLabelRequest::SetSystemLabel { .. } => ToolRisk::Reversible,
    }
}

/// Niveau de risque des requêtes du domaine label settings.
pub(super) fn label_settings_risk(request: &LabelSettingsRequest) -> ToolRisk {
    match request {
        LabelSettingsRequest::ListLabels => ToolRisk::Read,
        LabelSettingsRequest::SetLabelSettings { .. }
        | LabelSettingsRequest::ReorderLabels { .. } => ToolRisk::Reversible,
    }
}

/// Niveau de risque des requêtes du domaine template.
pub(super) fn template_risk(request: &TemplateRequest) -> ToolRisk {
    match request {
        TemplateRequest::ListTemplates
        | TemplateRequest::GetTemplate { .. }
        | TemplateRequest::RenderTemplate { .. } => ToolRisk::Read,
        TemplateRequest::CreateTemplate { .. } | TemplateRequest::UpdateTemplate { .. } => {
            ToolRisk::Reversible
        }
        TemplateRequest::DeleteTemplate { .. } => ToolRisk::Confirmed,
    }
}

/// Niveau de risque des requêtes du domaine settings.
pub(super) fn settings_risk(request: &SettingsRequest) -> ToolRisk {
    match request {
        SettingsRequest::GetSettings => ToolRisk::Read,
        SettingsRequest::SetSettings { .. } => ToolRisk::Sensitive,
    }
}
