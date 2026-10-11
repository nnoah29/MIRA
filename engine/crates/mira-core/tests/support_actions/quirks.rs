//! Particularités de service fournies aux tests, sans nommer de service.

use mira_vocabulary::{ArchiveStrategy, MailboxModel, ProviderQuirks};

/// Service à dossiers, archivage par déplacement (valeurs par défaut)
pub fn folders_quirks() -> ProviderQuirks {
    ProviderQuirks::default()
}

/// Service à libellés, archivage par retrait de la réception
pub fn labels_quirks() -> ProviderQuirks {
    ProviderQuirks {
        mailbox_model: MailboxModel::Labels,
        archive: ArchiveStrategy::RemoveFromInbox,
        ..ProviderQuirks::default()
    }
}
