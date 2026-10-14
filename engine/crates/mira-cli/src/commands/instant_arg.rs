//! Lecture d'un instant saisi : millisecondes Unix, `AAAA-MM-JJ` ou `AAAA-MM-JJTHH:MM[:SS]` (UTC).

use mira_assembly::mira_vocabulary::{MiraError, Timestamp};

use crate::argument_error::invalid_argument;
use crate::civil_date::days_from_civil;

const MILLIS_PER_DAY: i64 = 86_400_000;

/// Erreur de saisie d'un instant, sans écho de la valeur.
fn invalid_instant() -> MiraError {
    invalid_argument(
        "instant invalide : attendu millisecondes Unix, AAAA-MM-JJ ou AAAA-MM-JJTHH:MM[:SS]",
    )
}

/// Découpe `text` selon `separator` en nombres, ou rien si un morceau n'est pas un nombre.
fn numbers(text: &str, separator: char) -> Option<Vec<i64>> {
    text.split(separator)
        .map(|part| part.parse::<i64>().ok())
        .collect()
}

/// Millisecondes écoulées entre minuit et l'heure `HH:MM[:SS]`.
fn parse_time(text: &str) -> Result<i64, MiraError> {
    let parts = numbers(text, ':').ok_or_else(invalid_instant)?;
    let (hours, minutes, seconds) = match parts.as_slice() {
        [hours, minutes] => (*hours, *minutes, 0),
        [hours, minutes, seconds] => (*hours, *minutes, *seconds),
        _ => return Err(invalid_instant()),
    };
    if !(0..24).contains(&hours) || !(0..60).contains(&minutes) || !(0..60).contains(&seconds) {
        return Err(invalid_instant());
    }
    Ok(((hours * 60 + minutes) * 60 + seconds) * 1_000)
}

/// Millisecondes Unix du minuit UTC de la date `AAAA-MM-JJ`.
fn parse_date(text: &str) -> Result<i64, MiraError> {
    let parts = numbers(text, '-').ok_or_else(invalid_instant)?;
    match parts.as_slice() {
        [year, month, day] if (1..=12).contains(month) && (1..=31).contains(day) => {
            Ok(days_from_civil(*year, *month, *day) * MILLIS_PER_DAY)
        }
        _ => Err(invalid_instant()),
    }
}

/// Lit un instant : entier de millisecondes Unix, date ou date et heure UTC.
pub fn parse_instant(text: &str) -> Result<Timestamp, MiraError> {
    if let Ok(millis) = text.parse::<i64>() {
        return Ok(Timestamp::from_millis(millis));
    }
    let (date, time) = text.split_once('T').unwrap_or((text, "00:00"));
    let day_millis = parse_date(date)?;
    let time_millis = parse_time(time.trim_end_matches('Z'))?;
    Ok(Timestamp::from_millis(day_millis + time_millis))
}
