//! Routage exhaustif d'un test élémentaire vers l'évaluateur de sa famille.

use super::context::MessageContext;
use super::{address_eval as addr, compare_eval as compare, content_eval as content};
use super::{date_eval as date, own_address_eval as own};
use mira_vocabulary::ConditionTest as Test;

/// Évalue un test élémentaire sur le message, sans tenir compte de l'inversion.
pub fn test_holds(test: &Test, ctx: &MessageContext) -> bool {
    match test {
        Test::Sender { field, matcher } => addr::sender(ctx, *field, matcher),
        #[rustfmt::skip]
        Test::Recipient { which, field, matcher } => addr::recipient(ctx, *which, *field, matcher),
        Test::ReplyTo { field, matcher } => addr::reply_to(ctx, *field, matcher),
        Test::MeInTo => own::me_in_to(ctx),
        Test::MeInCcOnly => own::me_in_cc_only(ctx),
        Test::RecipientCount { cmp, n } => compare::recipient_count(ctx, *cmp, *n),
        Test::SizeCompare { cmp, bytes } => compare::size(ctx, *cmp, *bytes),
        Test::Subject { matcher } => content::subject(ctx, matcher),
        Test::Preview { matcher } => content::preview(ctx, matcher),
        Test::HasAttachment => content::has_attachment(ctx),
        Test::AttachmentName { matcher } => content::attachment_name(ctx, matcher),
        Test::AttachmentType { matcher } => content::attachment_type(ctx, matcher),
        Test::ReceivedBefore { at } => date::received_before(ctx, *at),
        Test::ReceivedAfter { at } => date::received_after(ctx, *at),
        Test::OlderThanDays { days } => date::older_than_days(ctx, *days),
        Test::IsRead => ctx.summary.flags.seen,
        Test::IsFlagged => ctx.summary.flags.flagged,
        Test::IsAnswered => ctx.summary.flags.answered,
        Test::IsForwarded => ctx.summary.flags.forwarded,
        Test::IsNewsletter => ctx.summary.unsubscribe.is_some(),
        Test::IsReply => ctx.summary.in_reply_to.is_some(),
        Test::InExistingThread => ctx.in_existing_thread,
        Test::InMailbox { mailbox } => ctx.summary.mailboxes.contains(mailbox),
        Test::FromAccount { account } => ctx.summary.account == *account,
    }
}
