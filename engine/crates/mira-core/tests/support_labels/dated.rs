//! Courrier dont l'en-tête `Date` est calculé à partir de l'horloge réelle (les verdicts dépendent de l'âge).

use crate::support_sync::mail::now_epoch;

const DAY: i64 = 86_400;
const BOB: &str = "Bob <bob@example.org>";
const ALICE: &str = "alice@example.org";

/// Un courrier à construire : expéditeur, destinataire, objet, fil, âge en jours et corps
pub struct Dated<'a> {
    pub from: &'a str,
    pub to: &'a str,
    pub subject: &'a str,
    pub id: &'a str,
    pub parent: Option<&'a str>,
    pub days_ago: i64,
    pub body: &'a str,
}

impl Dated<'_> {
    /// Octets du courrier, avec en-têtes de fil si `parent` est donné
    pub fn raw(&self) -> Vec<u8> {
        let thread = self
            .parent
            .map(|p| format!("In-Reply-To: <{p}>\r\nReferences: <{p}>\r\n"))
            .unwrap_or_default();
        format!(
            "From: {}\r\nTo: {}\r\nSubject: {}\r\nMessage-ID: <{}>\r\n{thread}Date: {}\r\n\r\n{}\r\n",
            self.from,
            self.to,
            self.subject,
            self.id,
            rfc2822(now_epoch() - self.days_ago * DAY),
            self.body
        )
        .into_bytes()
    }
}

/// Date RFC 2822 en UTC d'un instant Unix en secondes
pub fn rfc2822(epoch: i64) -> String {
    let days = epoch.div_euclid(DAY);
    let secs = epoch.rem_euclid(DAY);
    let (year, month, day) = civil_from_days(days);
    let weekday = ["Thu", "Fri", "Sat", "Sun", "Mon", "Tue", "Wed"][days.rem_euclid(7) as usize];
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    format!(
        "{weekday}, {day:02} {} {year} {:02}:{:02}:{:02} +0000",
        months[(month - 1) as usize],
        secs / 3600,
        secs % 3600 / 60,
        secs % 60
    )
}

/// Jour civil (année, mois, jour) d'un nombre de jours depuis 1970-01-01
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(month <= 2), month, day)
}

/// Message de Bob reçu il y a `days` jours
pub fn from_bob(subject: &str, id: &str, days: i64, body: &str) -> Vec<u8> {
    let parent = None;
    Dated {
        from: BOB,
        to: ALICE,
        subject,
        id,
        parent,
        days_ago: days,
        body,
    }
    .raw()
}

/// Ma réponse à `parent`, envoyée il y a `days` jours à Bob
pub fn my_reply(subject: &str, id: &str, parent: &str, days: i64) -> Vec<u8> {
    let (from, to) = (ALICE, "bob@example.org");
    let subject = format!("Re: {subject}");
    Dated {
        from,
        to,
        subject: &subject,
        id,
        parent: Some(parent),
        days_ago: days,
        body: "Voilà",
    }
    .raw()
}

/// Réponse de Bob à `parent`, reçue il y a `days` jours
pub fn bob_reply(subject: &str, id: &str, parent: &str, days: i64, body: &str) -> Vec<u8> {
    let subject = format!("Re: {subject}");
    Dated {
        from: BOB,
        to: ALICE,
        subject: &subject,
        id,
        parent: Some(parent),
        days_ago: days,
        body,
    }
    .raw()
}
