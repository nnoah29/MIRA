//! Vue Paramètres : comptes, identités, maintenance et journal d'événements.

pub mod account_actions;
pub mod account_nav;
pub mod account_submit;
pub mod events_log;
pub mod events_reply;
pub mod export_submit;
pub mod follow_up;
pub mod form_input;
pub mod identity_actions;
pub mod identity_submit;
pub mod intent;
pub mod maintenance;
pub mod maintenance_ask;
pub mod open;
pub mod operation;
pub mod operation_reply;
pub mod reply;
pub mod route;

pub use intent::{AccountOp, FormOp, IdentityOp, MaintenanceOp, PageOp, SettingsIntent};
