//! Extraction des filtres de texte d'une règle.

use mira_vocabulary::{ConditionTest, Rule, TextMatcher};

/// Filtre de texte porté par une condition, s'il y en a un.
fn matcher_of(test: &ConditionTest) -> Option<&TextMatcher> {
    match test {
        ConditionTest::Sender { matcher, .. }
        | ConditionTest::Recipient { matcher, .. }
        | ConditionTest::ReplyTo { matcher, .. }
        | ConditionTest::Subject { matcher }
        | ConditionTest::Preview { matcher }
        | ConditionTest::AttachmentName { matcher }
        | ConditionTest::AttachmentType { matcher } => Some(matcher),
        _ => None,
    }
}

/// Tous les filtres de texte de toutes les alternatives de la règle.
pub(super) fn rule_matchers(rule: &Rule) -> Vec<&TextMatcher> {
    rule.alternatives
        .iter()
        .flat_map(|group| group.conditions.iter())
        .filter_map(|condition| matcher_of(&condition.test))
        .collect()
}
