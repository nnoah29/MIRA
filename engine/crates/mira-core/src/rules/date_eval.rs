//! Conditions de date : avant, après, plus vieux que N jours.

use super::context::MessageContext;
use mira_vocabulary::Timestamp;

/// Millisecondes dans une journée.
const DAY_MILLIS: i64 = 86_400_000;

/// Le message est daté strictement avant l'instant donné.
pub fn received_before(ctx: &MessageContext, at: Timestamp) -> bool {
    message_date(ctx).is_some_and(|date| date < at)
}

/// Le message est daté strictement après l'instant donné.
pub fn received_after(ctx: &MessageContext, at: Timestamp) -> bool {
    message_date(ctx).is_some_and(|date| date > at)
}

/// Le message a au moins `days` jours à l'instant d'évaluation.
pub fn older_than_days(ctx: &MessageContext, days: u32) -> bool {
    let limit = i64::from(days).saturating_mul(DAY_MILLIS);
    let now = ctx.now.as_millis();
    message_date(ctx).is_some_and(|date| now.saturating_sub(date.as_millis()) >= limit)
}

/// Date de l'en-tête, ou date interne à défaut.
fn message_date(ctx: &MessageContext) -> Option<Timestamp> {
    ctx.summary.date.or(ctx.summary.internal_date)
}
