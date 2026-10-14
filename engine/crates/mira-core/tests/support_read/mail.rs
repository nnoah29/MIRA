//! Courriers bruts déposés sur le faux serveur : texte simple, HTML avec image distante, pièce jointe.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;

/// Courrier texte simple de sujet donné
pub fn plain(subject: &str, body: &str) -> Vec<u8> {
    format!("From: Bob <bob@example.org>\r\nTo: alice@folders.test\r\nSubject: {subject}\r\nMessage-ID: <{subject}@mira>\r\n\r\n{body}\r\n").into_bytes()
}

/// Courrier HTML dont l'image est hébergée sur un serveur distant
pub fn html_with_remote_image() -> Vec<u8> {
    let html = "<p>Bonjour</p><img src=\"https://tracker.example/pixel.gif\">";
    format!("From: Bob <bob@example.org>\r\nTo: alice@folders.test\r\nSubject: Promo\r\nMessage-ID: <promo@mira>\r\nMIME-Version: 1.0\r\nContent-Type: text/html; charset=utf-8\r\n\r\n{html}\r\n").into_bytes()
}

/// Courrier avec une pièce jointe binaire de `size` octets, encodée en base64
pub fn with_attachment(size: usize) -> Vec<u8> {
    let encoded = STANDARD.encode(vec![7u8; size]);
    let wrapped: Vec<&str> = encoded
        .as_bytes()
        .chunks(76)
        .map(|line| std::str::from_utf8(line).unwrap())
        .collect();
    let head = "From: Bob <bob@example.org>\r\nTo: alice@folders.test\r\nSubject: Rapport\r\nMessage-ID: <rapport@mira>\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary=\"B\"\r\n\r\n";
    let text = "--B\r\nContent-Type: text/plain\r\n\r\nVoir le fichier joint\r\n";
    let file = "--B\r\nContent-Type: application/octet-stream; name=\"data.bin\"\r\nContent-Disposition: attachment; filename=\"data.bin\"\r\nContent-Transfer-Encoding: base64\r\n\r\n";
    format!("{head}{text}{file}{}\r\n--B--\r\n", wrapped.join("\r\n")).into_bytes()
}
