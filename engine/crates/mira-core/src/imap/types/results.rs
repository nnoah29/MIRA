//! Résultats des opérations de haut niveau, sans syntaxe IMAP.

use mira_vocabulary::{RawMessageAttributes, Timestamp};

/// État connu d'une boîte, pour demander uniquement ce qui a changé (QRESYNC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResyncFrom {
    pub uid_validity: u32,
    pub highest_modseq: u64,
}

/// Drapeaux d'un message identifié par son UID.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedFlags {
    pub uid: u32,
    pub flags: Vec<String>,
    pub modseq: Option<u64>,
}

/// Boîte ouverte : marqueurs de synchronisation et changements depuis `ResyncFrom`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SelectedMailbox {
    pub uid_validity: u32,
    pub uid_next: u32,
    pub highest_modseq: Option<u64>,
    pub exists: u32,
    pub flags: Vec<String>,
    pub vanished: Vec<u32>,
    pub vanished_incomplete: bool,
    pub changed: Vec<FetchedFlags>,
}

/// En-têtes bruts d'un message avec ses métadonnées serveur.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedHeaders {
    pub uid: u32,
    pub flags: Vec<String>,
    pub size: u64,
    pub internal_date: Option<Timestamp>,
    pub modseq: Option<u64>,
    pub header_bytes: Vec<u8>,
    pub extra: RawMessageAttributes,
}

/// Issue d'une attente IDLE.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdleOutcome {
    Changed,
    TimedOut,
}
