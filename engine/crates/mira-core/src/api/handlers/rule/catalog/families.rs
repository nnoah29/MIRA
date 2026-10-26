//! Famille de chaque variante : un `match` exhaustif, une nouvelle variante ne compile pas sans être classée.

use mira_vocabulary::{ConditionTest, RuleAction};

/// Famille d'une condition (spec R1 §2.1)
pub(super) fn condition_family(test: &ConditionTest) -> &'static str {
    use ConditionTest as T;
    match test {
        T::Sender { .. } => "sender",
        T::Recipient { .. } | T::MeInTo | T::MeInCcOnly | T::RecipientCount { .. } => "recipients",
        T::ReplyTo { .. } => "reply_to",
        T::Subject { .. } | T::Preview { .. } => "content",
        T::ReceivedBefore { .. } | T::ReceivedAfter { .. } | T::OlderThanDays { .. } => "dates",
        T::SizeCompare { .. } => "size",
        T::HasAttachment | T::AttachmentName { .. } | T::AttachmentType { .. } => "attachments",
        T::IsRead | T::IsFlagged | T::IsAnswered | T::IsForwarded => "flags",
        T::IsNewsletter | T::IsReply | T::InExistingThread => "nature",
        T::InMailbox { .. } | T::FromAccount { .. } => "location",
        T::HasLocalLabel { .. } => "local_labels",
    }
}

/// Famille d'une action (spec R1 §2.2)
pub(super) fn action_family(action: &RuleAction) -> &'static str {
    use RuleAction as A;
    match action {
        A::MarkRead { .. } | A::MarkFlagged { .. } => "flags",
        A::MoveTo { .. } | A::AddToMailbox { .. } | A::RemoveFromMailbox { .. } | A::Archive => {
            "filing"
        }
        A::Trash | A::MarkSpam { .. } => "cleanup",
        A::AddLocalLabel { .. } | A::RemoveLocalLabel { .. } => "local_labels",
        A::SuppressNotification => "notification",
        A::StopProcessing => "chaining",
        A::ReplyWithTemplate { .. } => "reply",
    }
}
