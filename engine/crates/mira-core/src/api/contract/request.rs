//! Requêtes du contrat, regroupées par domaine (12.4) ; une variante = une opération = un handler.

use mira_vocabulary::{EventFilter, OperationId};
use serde::{Deserialize, Serialize};

use super::account_requests::AccountRequest;
use super::auth_requests::AuthRequest;
use super::identity_requests::IdentityRequest;
use super::label_settings_requests::LabelSettingsRequest;
use super::local_label_requests::LocalLabelRequest;
use super::mailbox_requests::MailboxRequest;
use super::maintenance_requests::MaintenanceRequest;
use super::message_requests::MessageRequest;
use super::rule_requests::RuleRequest;
use super::search_requests::SearchRequest;
use super::settings_requests::SettingsRequest;
use super::sync_requests::SyncRequest;
use super::system_label_requests::SystemLabelRequest;
use super::template_requests::TemplateRequest;
use super::thread_requests::ThreadRequest;
use super::write::{ActionRequest, ComposeRequest, QueueRequest};

/// Toute requête qu'un client peut adresser au moteur.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "domain", content = "request", rename_all = "snake_case")]
pub enum Request {
    Engine(EngineRequest),
    Provider(ProviderRequest),
    Account(AccountRequest),
    Identity(IdentityRequest),
    Mailbox(MailboxRequest),
    Message(MessageRequest),
    Thread(ThreadRequest),
    Search(SearchRequest),
    Maintenance(MaintenanceRequest),
    Auth(AuthRequest),
    Operation(OperationRequest),
    Event(EventRequest),
    Action(ActionRequest),
    Compose(ComposeRequest),
    Queue(QueueRequest),
    Sync(SyncRequest),
    Rule(RuleRequest),
    LocalLabel(LocalLabelRequest),
    SystemLabel(SystemLabelRequest),
    LabelSettings(LabelSettingsRequest),
    Settings(SettingsRequest),
    Template(TemplateRequest),
}

/// Requêtes sur le moteur lui-même.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum EngineRequest {
    GetContractInfo,
    GetStatus,
}

/// Requêtes sur le catalogue des providers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum ProviderRequest {
    ListProviders,
    DetectProvider { address: String },
}

/// Requêtes sur les opérations longues.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum OperationRequest {
    GetOperation { operation: OperationId },
    CancelOperation { operation: OperationId },
}

/// Lecture des événements par numéro (clients sans abonnement).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum EventRequest {
    GetEvents { filter: EventFilter, limit: u32 },
}
