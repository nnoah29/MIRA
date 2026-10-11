//! Fil de conversation.

use serde::{Deserialize, Serialize};

use crate::address::entry::AddressEntry;
use crate::ids::kinds::{AccountId, ThreadId};
use crate::time::Timestamp;

/// Fil regroupant des messages liés, avec ses compteurs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Thread {
    pub id: ThreadId,
    pub account: AccountId,
    pub subject: String,
    pub message_count: u32,
    pub unread_count: u32,
    pub last_date: Option<Timestamp>,
    pub participants: Vec<AddressEntry>,
}
