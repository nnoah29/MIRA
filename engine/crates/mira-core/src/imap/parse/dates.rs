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

/// Décalage en secondes d'un fuseau `+hhmm` ou `-hhmm`, ou rien s'il est mal formé.
fn zone_offset(text: &str) -> Option<i64> {
    let sign = if text.starts_with('-') { -1 } else { 1 };
    let hours: i64 = text.get(1..3)?.parse().ok()?;
    let minutes: i64 = text.get(3..5)?.parse().ok()?;
    if hours > 23 || minutes > 59 {
        return None;
    }
    Some(sign * (hours * 3600 + minutes * 60))
}

/// Nombre de jours d'un mois, février bissextile compris.
fn days_in_month(year: i64, month: u32) -> u32 {
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Lit une date interne `jj-Mon-aaaa hh:mm:ss +zzzz` et la convertit en instant UTC ; toute valeur hors bornes donne `None`.
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
    let zone = parts.next().map_or(Some(0), zone_offset)?;
    let valid = (1900..=9999).contains(&year)
        && (1..=days_in_month(year, month)).contains(&day)
        && (0..24).contains(&hour)
        && (0..60).contains(&minute)
        && (0..=60).contains(&second);
    if !valid {
        return None;
    }
    let days = days_from_civil(year, month, day);
    let local = days
        .checked_mul(86_400)?
        .checked_add(hour * 3600 + minute * 60 + second)?;
    let millis = local.checked_sub(zone)?.checked_mul(1000)?;
    Some(Timestamp::from_millis(millis))
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
