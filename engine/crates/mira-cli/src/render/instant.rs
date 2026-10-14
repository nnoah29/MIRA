//! Rendu humain d'un instant : `AAAA-MM-JJ HH:MM` en UTC.

use mira_assembly::mira_vocabulary::Timestamp;

use crate::civil_date::civil_from_days;

const MILLIS_PER_DAY: i64 = 86_400_000;
const MILLIS_PER_MINUTE: i64 = 60_000;

/// Date et heure UTC d'un instant, à la minute.
pub fn format_instant(instant: Timestamp) -> String {
    let millis = instant.as_millis();
    let (year, month, day) = civil_from_days(millis.div_euclid(MILLIS_PER_DAY));
    let minute_of_day = millis.rem_euclid(MILLIS_PER_DAY) / MILLIS_PER_MINUTE;
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}",
        minute_of_day / 60,
        minute_of_day % 60
    )
}

/// Date et heure d'un instant facultatif, ou un tiret s'il est absent.
pub fn format_optional_instant(instant: Option<Timestamp>) -> String {
    instant.map_or_else(|| "-".to_string(), format_instant)
}
