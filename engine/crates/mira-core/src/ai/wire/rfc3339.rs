//! Conversion d'une date RFC 3339 en secondes Unix, sans dépendance de calendrier.

/// Convertit une date RFC 3339 (`2025-02-19T00:00:00Z`, fraction et décalage admis) en secondes Unix.
pub fn unix_seconds(text: &str) -> Option<i64> {
    let (date, rest) = text.split_once(['T', 't'])?;
    let (time, offset) = split_offset(rest)?;
    let days = days_since_epoch(date)?;
    Some(days * 86_400 + seconds_of_day(time)? - offset)
}

/// Sépare l'heure du décalage et renvoie ce dernier en secondes (Z vaut 0).
fn split_offset(rest: &str) -> Option<(&str, i64)> {
    if let Some(time) = rest.strip_suffix(['Z', 'z']) {
        return Some((time, 0));
    }
    let at = rest.rfind(['+', '-'])?;
    let (time, sign_offset) = rest.split_at(at);
    let (hours, minutes) = sign_offset.get(1..)?.split_once(':')?;
    let total = number(hours, 2)? * 3600 + number(minutes, 2)? * 60;
    Some((
        time,
        if sign_offset.starts_with('-') {
            -total
        } else {
            total
        },
    ))
}

/// Lit `hh:mm:ss` avec fraction facultative, qui est ignorée.
fn seconds_of_day(time: &str) -> Option<i64> {
    let whole = time.split('.').next()?;
    let mut parts = whole.split(':');
    let (hours, minutes, seconds) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() {
        return None;
    }
    let (h, m, s) = (number(hours, 2)?, number(minutes, 2)?, number(seconds, 2)?);
    (h < 24 && m < 60 && s <= 60).then_some(h * 3600 + m * 60 + s)
}

/// Compte les jours entre le 1er janvier 1970 et une date `aaaa-mm-jj` (algorithme du calendrier civil).
fn days_since_epoch(date: &str) -> Option<i64> {
    let mut parts = date.split('-');
    let (year, month, day) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() {
        return None;
    }
    let (y, m, d) = (number(year, 4)?, number(month, 2)?, number(day, 2)?);
    if !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let year_of_era = y - era * 400;
    let day_of_year = (153 * ((m + 9) % 12) + 2) / 5 + d - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Some(era * 146_097 + day_of_era - 719_468)
}

/// Lit un entier décimal de exactement `digits` chiffres.
fn number(text: &str, digits: usize) -> Option<i64> {
    if text.len() != digits || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}
