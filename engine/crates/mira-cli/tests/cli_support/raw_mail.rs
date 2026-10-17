//! Courriers bruts déposés sur le faux serveur IMAP, alignés sur les messages du miroir garni.

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;

/// Taille de la pièce jointe du message m4.
pub const ATTACHMENT_SIZE: usize = 2_000;

/// Courrier texte simple `m<index>` dont le corps est donné.
pub fn plain(index: u32, body: &str) -> Vec<u8> {
    format!("From: Bob <bob@example.org>\r\nTo: alice@folders.test\r\nSubject: Sujet m{index}\r\nMessage-ID: <m{index}@example.org>\r\n\r\n{body}\r\n").into_bytes()
}

/// Courrier `m4` avec une pièce jointe binaire `data.bin` de `ATTACHMENT_SIZE` octets.
pub fn with_attachment() -> Vec<u8> {
    let encoded = STANDARD.encode(vec![7u8; ATTACHMENT_SIZE]);
    let wrapped: Vec<&str> = encoded
        .as_bytes()
        .chunks(76)
        .map(|line| std::str::from_utf8(line).unwrap())
        .collect();
    let head = "From: Carla <carla@example.org>\r\nTo: alice@folders.test\r\nSubject: Sujet m4\r\nMessage-ID: <m4@example.org>\r\nMIME-Version: 1.0\r\nContent-Type: multipart/mixed; boundary=\"B\"\r\n\r\n";
    let text = "--B\r\nContent-Type: text/plain\r\n\r\nVoir le fichier joint\r\n";
    let file = "--B\r\nContent-Type: application/octet-stream; name=\"data.bin\"\r\nContent-Disposition: attachment; filename=\"data.bin\"\r\nContent-Transfer-Encoding: base64\r\n\r\n";
    format!("{head}{text}{file}{}\r\n--B--\r\n", wrapped.join("\r\n")).into_bytes()
}
