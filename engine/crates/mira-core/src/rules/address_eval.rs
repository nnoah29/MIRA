//! Conditions sur l'expéditeur, l'adresse de réponse et les destinataires.

use super::context::MessageContext;
use super::text_match::matches_text;
use mira_vocabulary::{AddressEntry, AddressField, ConditionTest, RecipientKind, TextMatcher};

/// Résultat d'un test d'adresse (expéditeur, destinataire, réponse), ou rien pour un autre test.
pub fn holds(test: &ConditionTest, ctx: &MessageContext) -> Option<bool> {
    match test {
        ConditionTest::Sender { field, matcher } => Some(sender(ctx, *field, matcher)),
        ConditionTest::Recipient {
            which,
            field,
            matcher,
        } => Some(recipient(ctx, *which, *field, matcher)),
        ConditionTest::ReplyTo { field, matcher } => Some(reply_to(ctx, *field, matcher)),
        _ => None,
    }
}

/// L'expéditeur correspond au filtre sur le champ donné.
pub fn sender(ctx: &MessageContext, field: AddressField, matcher: &TextMatcher) -> bool {
    any_entry_matches(&ctx.summary.from, field, matcher)
}

/// L'adresse de réponse correspond au filtre sur le champ donné.
pub fn reply_to(ctx: &MessageContext, field: AddressField, matcher: &TextMatcher) -> bool {
    any_entry_matches(&ctx.summary.reply_to, field, matcher)
}

/// Un destinataire du rôle donné correspond au filtre sur le champ donné.
pub fn recipient(
    ctx: &MessageContext,
    which: RecipientKind,
    field: AddressField,
    matcher: &TextMatcher,
) -> bool {
    let entries = match which {
        RecipientKind::To => &ctx.summary.to,
        RecipientKind::Cc => &ctx.summary.cc,
        RecipientKind::Bcc => &ctx.summary.bcc,
    };
    any_entry_matches(entries, field, matcher)
}

/// Vrai si une adresse de la liste a un champ présent qui correspond au filtre.
fn any_entry_matches(entries: &[AddressEntry], field: AddressField, matcher: &TextMatcher) -> bool {
    entries
        .iter()
        .filter_map(|entry| field_value(entry, field))
        .any(|value| matches_text(matcher, value))
}

/// Extrait le champ demandé d'une adresse, ou rien s'il est absent ou vide.
fn field_value(entry: &AddressEntry, field: AddressField) -> Option<&str> {
    let value = match field {
        AddressField::Address => Some(entry.address.as_str()),
        AddressField::Name => entry.name.as_deref(),
        AddressField::Domain => entry.address.rsplit_once('@').map(|(_, domain)| domain),
    };
    value.filter(|text| !text.is_empty())
}
