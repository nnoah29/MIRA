//! Date courte d'une ligne de message, en UTC : `JJ/MM HH:MM`.

use mira_assembly::mira_vocabulary::Timestamp;

const MILLIS_PER_DAY: i64 = 86_400_000;

/// Formate un instant en `JJ/MM HH:MM` (UTC).
pub fn short_date(timestamp: Timestamp) -> String {
    let days = timestamp.millis.div_euclid(MILLIS_PER_DAY);
    let minutes = timestamp.millis.rem_euclid(MILLIS_PER_DAY) / 60_000;
    let (month, day) = month_and_day(days);
    format!(
        "{day:02}/{month:02} {:02}:{:02}",
        minutes / 60,
        minutes % 60
    )
}

/// Mois et jour du mois pour un nombre de jours depuis le 1970-01-01.
fn month_and_day(days_since_epoch: i64) -> (i64, i64) {
    let z = days_since_epoch + 719_468;
    let day_of_era = z.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    (month, day)
}
