//! Courrier d'exemple pour les tests IMAP.

/// Courrier d'exemple avec un identifiant de message connu.
pub const MAIL: &[u8] = b"From: Alice <alice@example.org>\r\nTo: bob@example.org\r\nSubject: Hello\r\nMessage-ID: <abc@mira>\r\n\r\nBody text\r\n";

/// Courrier d'exemple avec un sujet et un identifiant choisis
pub fn mail(subject: &str, message_id: &str) -> Vec<u8> {
    format!("From: Alice <alice@example.org>\r\nSubject: {subject}\r\nMessage-ID: <{message_id}>\r\n\r\nBody of {subject}\r\n").into_bytes()
}
