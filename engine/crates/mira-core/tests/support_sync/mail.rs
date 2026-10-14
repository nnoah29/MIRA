//! Courrier de test avec identifiant de message, date et corps distincts.

/// Courrier avec objet, identifiant de message et date d'en-tête fixes
pub fn mail(subject: &str, message_id: &str) -> Vec<u8> {
    format!(
        "From: Bob <bob@example.org>\r\nTo: alice@example.org\r\nSubject: {subject}\r\nMessage-ID: <{message_id}>\r\nDate: Mon, 01 Jan 2024 10:00:00 +0000\r\n\r\nCorps de {subject}\r\n"
    )
    .into_bytes()
}

/// Courrier qui répond à un autre (en-têtes de fil)
pub fn reply(subject: &str, message_id: &str, parent_id: &str) -> Vec<u8> {
    format!(
        "From: Bob <bob@example.org>\r\nTo: alice@example.org\r\nSubject: Re: {subject}\r\nMessage-ID: <{message_id}>\r\nIn-Reply-To: <{parent_id}>\r\nReferences: <{parent_id}>\r\nDate: Mon, 01 Jan 2024 11:00:00 +0000\r\n\r\nRéponse\r\n"
    )
    .into_bytes()
}

/// Secondes Unix actuelles
pub fn now_epoch() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Secondes Unix d'il y a `days` jours
pub fn days_ago(days: i64) -> i64 {
    now_epoch() - days * 86_400
}
