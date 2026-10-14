//! Réponses du contrat : une variante par forme de données rendue.

use mira_vocabulary::{
    Account, Attachment, ContractVersion, Event, Identity, Mailbox, MessageSummary, Page,
    ProfileId, ProviderDescriptor, ServerSettings, Thread,
};
use serde::{Deserialize, Serialize};

use super::read_responses::{ExportReport, IntegrityReport, RawSourceView, ThreadView};
use super::sync_response::SyncResponse;
use super::write::WriteResponse;
use crate::fetching::ServerSearchResult;
use crate::html::SafeHtml;
use crate::operations::OperationInfo;
use crate::reading::BodyView;
use crate::storage::repositories::maintenance::StorageStats;

/// Version du contrat et capacités du moteur (12.4, Moteur).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractInfo {
    pub version: ContractVersion,
    pub engine_capabilities: Vec<String>,
}

/// État global du moteur : profil, emplacement des données, comptes et actions en attente.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EngineStatus {
    pub profile: ProfileId,
    pub data_directory: String,
    pub accounts: Vec<AccountStatus>,
}

/// Compte et nombre de ses actions encore en attente.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountStatus {
    pub account: Account,
    pub pending_actions: u64,
}

/// Préparation d'un compte : provider retenu et serveurs proposés (S1–S3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountPreparation {
    pub provider: ProviderDescriptor,
    pub servers: ServerSettings,
}

/// Début d'un flux OAuth : identifiant du flux et adresse à ouvrir dans le navigateur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OAuthStart {
    pub flow: String,
    pub authorize_url: String,
}

/// Toute réponse du moteur.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum Response {
    Done,
    ContractInfo(ContractInfo),
    Status(EngineStatus),
    Providers(Vec<ProviderDescriptor>),
    Provider(ProviderDescriptor),
    AccountPreparation(AccountPreparation),
    Accounts(Vec<Account>),
    Account(Account),
    Identities(Vec<Identity>),
    Identity(Identity),
    Mailboxes(Vec<Mailbox>),
    Messages(Page<MessageSummary>),
    Message(MessageSummary),
    MessageBody(BodyView),
    RawSource(RawSourceView),
    Attachments(Vec<Attachment>),
    SafeHtml(SafeHtml),
    Threads(Page<Thread>),
    Thread(ThreadView),
    ServerSearch(ServerSearchResult),
    StorageStats(StorageStats),
    Integrity(IntegrityReport),
    Exported(ExportReport),
    #[serde(rename = "oauth_started")]
    OAuthStarted(OAuthStart),
    Operation(OperationInfo),
    Events(Vec<Event>),
    Write(WriteResponse),
    Sync(SyncResponse),
}
