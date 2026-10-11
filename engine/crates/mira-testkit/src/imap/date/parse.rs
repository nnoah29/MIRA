//! Lecture des dates IMAP (critères de recherche et dates de APPEND).

use super::civil::days_from_civil;
use super::months::month_number;

/// Jours depuis l'époque pour une date `d-Mon-yyyy`
pub(crate) fn parse_day(text: &str) -> Option<i64> {
    let mut parts = text.trim().split('-');
    let day: u32 = parts.next()?.trim().parse().ok()?;
    let month = month_number(parts.next()?)?;
    let year: i64 = parts.next()?.parse().ok()?;
    Some(days_from_civil(year, month, day))
}

/// Secondes Unix pour une date `d-Mon-yyyy hh:mm:ss +zzzz`
pub(crate) fn parse_internal_date(text: &str) -> Option<i64> {
    let mut parts = text.split_whitespace();
    let days = parse_day(parts.next()?)?;
    let mut clock = parts.next()?.split(':');
    let hour: i64 = clock.next()?.parse().ok()?;
    let minute: i64 = clock.next()?.parse().ok()?;
    let second: i64 = clock.next()?.parse().ok()?;
    let zone = parts.next().and_then(parse_zone).unwrap_or(0);
    Some(days * 86_400 + hour * 3600 + minute * 60 + second - zone)
}

/// Décalage en secondes d'un fuseau `+hhmm` ou `-hhmm`
fn parse_zone(text: &str) -> Option<i64> {
    let sign = if text.starts_with('-') { -1 } else { 1 };
    let digits = text.get(1..5)?;
    let hours: i64 = digits.get(..2)?.parse().ok()?;
    let minutes: i64 = digits.get(2..)?.parse().ok()?;
    Some(sign * (hours * 3600 + minutes * 60))
}
