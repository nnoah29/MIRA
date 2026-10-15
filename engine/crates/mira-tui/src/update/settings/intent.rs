//! Intentions de la vue Paramètres, groupées par page.

use crate::state::settings::SettingsTab;

/// Ce que l'utilisateur fait dans la vue Paramètres.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsIntent {
    Show(SettingsTab),
    NextTab,
    PrevTab,
    Account(AccountOp),
    Identity(IdentityOp),
    Maintenance(MaintenanceOp),
    Page(PageOp),
    Form(FormOp),
}

/// Opérations de la page Comptes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountOp {
    Edit,
    Pause,
    Resume,
    Delete,
}

/// Opérations de la page Identités.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityOp {
    Add,
    Edit,
    SetDefault,
}

/// Opérations de la page Maintenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaintenanceOp {
    Stats,
    Check,
    Purge,
    Rebuild,
    ExportDiagnostic,
    ExportProfile,
}

/// Changements de page du journal d'événements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageOp {
    Next,
    Previous,
    First,
    Last,
}

/// Saisie dans le formulaire ouvert.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormOp {
    Char(char),
    Backspace,
    NextField,
    PrevField,
    Submit,
}
