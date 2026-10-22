//! Catalogue des conditions élémentaires d'une règle.

use serde::{Deserialize, Serialize};

use super::matcher::{AddressField, Comparison, RecipientKind, TextMatcher};
use crate::ids::kinds::{AccountId, LocalLabelId, MailboxId};
use crate::time::Timestamp;

/// Test élémentaire porté par une condition.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ConditionTest {
    Sender {
        field: AddressField,
        matcher: TextMatcher,
    },
    Recipient {
        which: RecipientKind,
        field: AddressField,
        matcher: TextMatcher,
    },
    MeInTo,
    MeInCcOnly,
    RecipientCount {
        cmp: Comparison,
        n: u32,
    },
    ReplyTo {
        field: AddressField,
        matcher: TextMatcher,
    },
    Subject {
        matcher: TextMatcher,
    },
    Preview {
        matcher: TextMatcher,
    },
    ReceivedBefore {
        at: Timestamp,
    },
    ReceivedAfter {
        at: Timestamp,
    },
    OlderThanDays {
        days: u32,
    },
    SizeCompare {
        cmp: Comparison,
        bytes: u64,
    },
    HasAttachment,
    AttachmentName {
        matcher: TextMatcher,
    },
    AttachmentType {
        matcher: TextMatcher,
    },
    IsRead,
    IsFlagged,
    IsAnswered,
    IsForwarded,
    IsNewsletter,
    IsReply,
    InExistingThread,
    InMailbox {
        mailbox: MailboxId,
    },
    FromAccount {
        account: AccountId,
    },
    HasLocalLabel {
        label: LocalLabelId,
    },
}
