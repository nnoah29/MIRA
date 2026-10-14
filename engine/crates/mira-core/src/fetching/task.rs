//! Contexte d'une tâche réseau de lecture : accès au serveur, base, identifiants, événements, annulation.

use std::path::PathBuf;
use std::sync::Arc;

use super::access::ImapAccess;
use crate::events::EventSink;
use crate::operations::OperationHandle;
use crate::ports::IdGenerator;
use crate::storage::Database;

/// Ce qu'emporte une tâche longue lancée en arrière-plan ; tout est partageable et `'static`.
#[derive(Debug, Clone)]
pub struct FetchTask {
    pub access: ImapAccess,
    pub database: Arc<Database>,
    pub ids: Arc<dyn IdGenerator>,
    pub events: Arc<dyn EventSink>,
    pub handle: OperationHandle,
    pub cache_directory: PathBuf,
}
