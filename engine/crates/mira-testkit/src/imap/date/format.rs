//! Formatage d'une date interne au format IMAP.

use super::civil::civil_from_days;
use super::months::month_name;

/// Formate des secondes Unix en `dd-Mon-yyyy hh:mm:ss +0000`
pub(crate) fn format_internal_date(epoch: i64) -> String {
    let (year, month, day) = civil_from_days(epoch.div_euclid(86_400));
    let secs = epoch.rem_euclid(86_400);
    let (hour, minute, second) = (secs / 3600, secs % 3600 / 60, secs % 60);
    let name = month_name(month);
    format!("{day:02}-{name}-{year} {hour:02}:{minute:02}:{second:02} +0000")
}
