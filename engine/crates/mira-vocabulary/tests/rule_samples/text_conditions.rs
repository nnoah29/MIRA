//! Exemples de conditions à filtre texte.

use mira_vocabulary::{AddressField, ConditionTest, RecipientKind, TextMatcher, TextOp};

/// Construit un filtre texte « contient » sur une valeur.
fn contains(value: &str) -> TextMatcher {
    TextMatcher {
        op: TextOp::Contains,
        values: vec![value.to_string()],
    }
}

/// Liste une instance de chaque condition à filtre texte.
pub fn text_conditions() -> Vec<ConditionTest> {
    vec![
        ConditionTest::Sender {
            field: AddressField::Domain,
            matcher: contains("a"),
        },
        ConditionTest::Recipient {
            which: RecipientKind::Bcc,
            field: AddressField::Name,
            matcher: contains("b"),
        },
        ConditionTest::ReplyTo {
            field: AddressField::Address,
            matcher: contains("c"),
        },
        ConditionTest::Subject {
            matcher: contains("d"),
        },
        ConditionTest::Preview {
            matcher: contains("e"),
        },
        ConditionTest::AttachmentName {
            matcher: contains("f"),
        },
        ConditionTest::AttachmentType {
            matcher: contains("g"),
        },
    ]
}
