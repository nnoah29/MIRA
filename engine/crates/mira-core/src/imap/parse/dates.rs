//! Dates IMAP : date interne d'un message et date de critère de recherche.

use mira_vocabulary::Timestamp;

use super::civil::{civil_from_days, days_from_civil};

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Numéro (1 à 12) d'un mois abrégé en anglais.
fn month_number(name: &str) -> Option<u32> {
    let index = MONTHS
        .iter()
        .position(|month| month.eq_ignore_ascii_case(name))?;
    Some(index as u32 + 1)
}

/// Décalage en secondes d'un fuseau `+hhmm` ou `-hhmm`.
fn zone_offset(text: &str) -> Option<i64> {
    let sign = if text.starts_with('-') { -1 } else { 1 };
    let hours: i64 = text.get(1..3)?.parse().ok()?;
    let minutes: i64 = text.get(3..5)?.parse().ok()?;
    Some(sign * (hours * 3600 + minutes * 60))
}

/// Lit une date interne `jj-Mon-aaaa hh:mm:ss +zzzz` et la convertit en instant UTC.
pub(crate) fn parse_internal_date(text: &str) -> Option<Timestamp> {
    let mut parts = text.split_whitespace();
    let mut day_part = parts.next()?.split('-');
    let day: u32 = day_part.next()?.parse().ok()?;
    let month = month_number(day_part.next()?)?;
    let year: i64 = day_part.next()?.parse().ok()?;
    let mut clock = parts.next()?.split(':');
    let hour: i64 = clock.next()?.parse().ok()?;
    let minute: i64 = clock.next()?.parse().ok()?;
    let second: i64 = clock.next()?.parse().ok()?;
    let zone = parts.next().and_then(zone_offset).unwrap_or(0);
    let seconds = days_from_civil(year, month, day) * 86_400 + hour * 3600 + minute * 60 + second;
    Some(Timestamp::from_millis((seconds - zone) * 1000))
}

/// Écrit un instant comme date de critère de recherche (`1-Jan-2023`), en UTC.
pub(crate) fn format_search_date(instant: Timestamp) -> String {
    let days = instant.as_millis().div_euclid(86_400_000);
    let (year, month, day) = civil_from_days(days);
    format!(
        "{day}-{}-{year}",
        MONTHS[(month as usize).saturating_sub(1) % 12]
    )
}
