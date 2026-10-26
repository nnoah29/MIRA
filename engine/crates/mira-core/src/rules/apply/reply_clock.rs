//! Date et heure UTC du moment de la règle, au format du rendu des modèles (`jj/mm/aaaa`, `hh:mm`).

use mira_vocabulary::Timestamp;

const MILLIS_PER_DAY: i64 = 86_400_000;

/// Jour civil (année, mois, jour) d'un nombre de jours depuis le 1er janvier 1970
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day_of_era = shifted.rem_euclid(146_097);
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

/// Date `jj/mm/aaaa` et heure `hh:mm` de l'instant, en UTC
pub(super) fn date_and_time(now: Timestamp) -> (String, String) {
    let (year, month, day) = civil_from_days(now.millis.div_euclid(MILLIS_PER_DAY));
    let minutes = now.millis.rem_euclid(MILLIS_PER_DAY) / 60_000;
    (
        format!("{day:02}/{month:02}/{year}"),
        format!("{:02}:{:02}", minutes / 60, minutes % 60),
    )
}
