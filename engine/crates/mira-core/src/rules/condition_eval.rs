//! Routage exhaustif d'un test élémentaire vers l'évaluateur de sa famille.

use super::context::MessageContext;
use super::{address_eval as addr, compare_eval as compare, content_eval as content};
use super::{date_eval as date, own_address_eval as own};
use mira_vocabulary::ConditionTest;

/// Évalue un test élémentaire sur le message, sans tenir compte de l'inversion.
pub fn test_holds(test: &ConditionTest, ctx: &MessageContext) -> bool {
    match test {
        ConditionTest::Sender { .. }
        | ConditionTest::Recipient { .. }
        | ConditionTest::ReplyTo { .. } => addr::holds(test, ctx).unwrap_or(false),
        ConditionTest::MeInTo => own::me_in_to(ctx),
        ConditionTest::MeInCcOnly => own::me_in_cc_only(ctx),
        ConditionTest::RecipientCount { cmp, n } => compare::recipient_count(ctx, *cmp, *n),
        ConditionTest::SizeCompare { cmp, bytes } => compare::size(ctx, *cmp, *bytes),
        ConditionTest::Subject { matcher } => content::subject(ctx, matcher),
        ConditionTest::Preview { matcher } => content::preview(ctx, matcher),
        ConditionTest::HasAttachment => content::has_attachment(ctx),
        ConditionTest::AttachmentName { matcher } => content::attachment_name(ctx, matcher),
        ConditionTest::AttachmentType { matcher } => content::attachment_type(ctx, matcher),
        ConditionTest::ReceivedBefore { at } => date::received_before(ctx, *at),
        ConditionTest::ReceivedAfter { at } => date::received_after(ctx, *at),
        ConditionTest::OlderThanDays { days } => date::older_than_days(ctx, *days),
        ConditionTest::IsRead => ctx.summary.flags.seen,
        ConditionTest::IsFlagged => ctx.summary.flags.flagged,
        ConditionTest::IsAnswered => ctx.summary.flags.answered,
        ConditionTest::IsForwarded => ctx.summary.flags.forwarded,
        ConditionTest::IsNewsletter => ctx.summary.unsubscribe.is_some(),
        ConditionTest::IsReply => ctx.summary.in_reply_to.is_some(),
        ConditionTest::InExistingThread => ctx.in_existing_thread,
        ConditionTest::InMailbox { mailbox } => ctx.summary.mailboxes.contains(mailbox),
        ConditionTest::FromAccount { account } => ctx.summary.account == *account,
    }
}
