//! Courrier daté dans le passé et horloge réelle, pour les scénarios de plage de synchronisation.

use std::time::{SystemTime, UNIX_EPOCH};

use mira_assembly::mira_vocabulary::Timestamp;

use super::sb_world::SbWorld;

/// Secondes Unix d'il y a `days` jours.
fn seconds_ago(days: i64) -> i64 {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    i64::try_from(now.as_secs()).unwrap() - days * 86_400
}

impl SbWorld {
    /// Fait indiquer à l'horloge manuelle l'heure réelle (les dates du faux serveur sont réelles).
    pub fn use_real_time(&self) {
        let now = seconds_ago(0) * 1_000;
        self.clock.set(Timestamp::from_millis(now));
    }

    /// Dépose dans la réception un courrier daté d'il y a `age` jours, éventuellement en réponse à un autre.
    pub fn deliver_aged(&self, user: &str, subject: &str, age: i64, reply_to: Option<&str>) {
        let parent = reply_to.map_or(String::new(), |id| {
            format!("In-Reply-To: <{id}@example.org>\r\n")
        });
        let raw = format!(
            "From: Bob <bob@example.org>\r\nTo: {user}\r\nSubject: {subject}\r\nMessage-ID: <{subject}@example.org>\r\n{parent}\r\nCorps de {subject}\r\n"
        );
        let date = seconds_ago(age);
        self.store
            .deliver_dated(user, "INBOX", raw.as_bytes(), &[], date);
    }
}
