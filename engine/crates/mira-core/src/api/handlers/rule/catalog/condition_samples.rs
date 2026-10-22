//! Une instance de chaque variante de condition, dont le catalogue lit l'étiquette et les paramètres.

use mira_vocabulary::{
    AccountId, AddressField, Comparison, ConditionTest, LocalLabelId, MailboxId, RecipientKind,
    TextMatcher, TextOp, Timestamp,
};

/// Filtre texte vide servant d'exemple
fn sample_matcher() -> TextMatcher {
    TextMatcher {
        op: TextOp::Contains,
        values: Vec::new(),
    }
}

/// Conditions sur les adresses : expéditeur, destinataires, « répondre à »
fn address_samples() -> Vec<ConditionTest> {
    vec![
        ConditionTest::Sender {
            field: AddressField::Address,
            matcher: sample_matcher(),
        },
        ConditionTest::Recipient {
            which: RecipientKind::To,
            field: AddressField::Address,
            matcher: sample_matcher(),
        },
        ConditionTest::MeInTo,
        ConditionTest::MeInCcOnly,
        ConditionTest::RecipientCount {
            cmp: Comparison::Greater,
            n: 0,
        },
        ConditionTest::ReplyTo {
            field: AddressField::Address,
            matcher: sample_matcher(),
        },
    ]
}

/// Conditions sur le contenu, les dates, la taille et les pièces jointes
fn content_samples() -> Vec<ConditionTest> {
    vec![
        ConditionTest::Subject {
            matcher: sample_matcher(),
        },
        ConditionTest::Preview {
            matcher: sample_matcher(),
        },
        ConditionTest::ReceivedBefore {
            at: Timestamp::from_millis(0),
        },
        ConditionTest::ReceivedAfter {
            at: Timestamp::from_millis(0),
        },
        ConditionTest::OlderThanDays { days: 0 },
        ConditionTest::SizeCompare {
            cmp: Comparison::Greater,
            bytes: 0,
        },
        ConditionTest::HasAttachment,
        ConditionTest::AttachmentName {
            matcher: sample_matcher(),
        },
        ConditionTest::AttachmentType {
            matcher: sample_matcher(),
        },
    ]
}

/// Conditions sur les drapeaux, la nature du message et son emplacement
fn state_samples() -> Vec<ConditionTest> {
    vec![
        ConditionTest::IsRead,
        ConditionTest::IsFlagged,
        ConditionTest::IsAnswered,
        ConditionTest::IsForwarded,
        ConditionTest::IsNewsletter,
        ConditionTest::IsReply,
        ConditionTest::InExistingThread,
        ConditionTest::InMailbox {
            mailbox: MailboxId::from_raw(""),
        },
        ConditionTest::FromAccount {
            account: AccountId::from_raw(""),
        },
        ConditionTest::HasLocalLabel {
            label: LocalLabelId::from_raw(""),
        },
    ]
}

/// Une instance de chaque variante de condition, dans l'ordre du catalogue
pub(super) fn condition_samples() -> Vec<ConditionTest> {
    let mut all = address_samples();
    all.extend(content_samples());
    all.extend(state_samples());
    all
}
