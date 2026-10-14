//! Valeurs d'exemple partagées par les réponses de lecture et de maintenance.

use mira_core::api::contract::ExportReport;
use mira_core::fetching::{ServerSearchHit, ServerSearchResult};
use mira_core::storage::repositories::maintenance::StorageStats;
use mira_vocabulary::{
    AccountId, AddressEntry, Attachment, AttachmentId, CacheState, MailboxId, MessageId, Thread,
    ThreadId, Timestamp,
};

/// Pièce jointe d'exemple, non téléchargée
pub fn sample_attachment() -> Attachment {
    Attachment {
        id: AttachmentId::from_raw("t"),
        message: MessageId::from_raw("m"),
        filename: Some("a.txt".to_string()),
        mime_type: "text/plain".to_string(),
        size: 3,
        content_id: None,
        inline: false,
        cache: CacheState::NotDownloaded,
    }
}

/// Fil d'exemple d'un seul message
pub fn sample_thread() -> Thread {
    Thread {
        id: ThreadId::from_raw("t"),
        account: AccountId::from_raw("a"),
        subject: "Sujet".to_string(),
        message_count: 1,
        unread_count: 1,
        last_date: Some(Timestamp::from_millis(5)),
        participants: vec![AddressEntry {
            name: None,
            address: "a@b.c".to_string(),
        }],
    }
}

/// Volumes d'exemple du miroir
pub fn sample_stats() -> StorageStats {
    StorageStats {
        mailboxes: 1,
        messages: 2,
        appearances: 2,
        threads: 1,
        raw_sources: 1,
        raw_bytes: 10,
        compressed_bytes: 8,
        attachments: 1,
        attachments_cached: 0,
    }
}

/// Résultat d'une recherche serveur à un seul message trouvé
pub fn sample_search_result() -> ServerSearchResult {
    let hit = ServerSearchHit {
        mailbox: MailboxId::from_raw("b"),
        uid: 7,
        message: None,
    };
    ServerSearchResult {
        hits: vec![hit],
        truncated: false,
    }
}

/// Rapport d'un export écrit
pub fn sample_export() -> ExportReport {
    ExportReport {
        path: "/tmp/profile.json".to_string(),
        bytes: 120,
    }
}
