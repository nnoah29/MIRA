//! Courriers et identifiants des faux serveurs : messages bruts texte et HTML.

pub const USER: &str = "alice@bridge.test";
pub const PASSWORD: &str = "pa55-SECRET-xyz";

/// Message brut minimal, expédié par Bob.
pub fn raw_mail(subject: &str, body: &str) -> Vec<u8> {
    format!(
        "From: Bob <bob@example.org>\r\nTo: Alice <{USER}>\r\nSubject: {subject}\r\n\
         Date: Mon, 01 Jan 2024 10:00:00 +0000\r\nMessage-ID: <{subject}@example.org>\r\n\
         Content-Type: text/plain; charset=utf-8\r\n\r\n{body}\r\n"
    )
    .into_bytes()
}

/// Message HTML avec une image incorporée (`cid:`) et une image distante.
pub fn html_mail() -> Vec<u8> {
    format!(
        "From: Bob <bob@example.org>\r\nTo: {USER}\r\nSubject: Riche\r\n\
         Date: Mon, 01 Jan 2024 11:00:00 +0000\r\nMessage-ID: <riche@example.org>\r\n\
         MIME-Version: 1.0\r\nContent-Type: multipart/related; boundary=\"B\"\r\n\r\n\
         --B\r\nContent-Type: text/html; charset=utf-8\r\n\r\n\
         <table><tr><td>Bonjour</td></tr></table><img src=\"cid:logo@x\">\
         <img src=\"https://tracker.example/p.png\">\r\n\
         --B\r\nContent-Type: image/png\r\nContent-ID: <logo@x>\r\n\
         Content-Disposition: inline; filename=\"logo.png\"\r\n\
         Content-Transfer-Encoding: base64\r\n\r\niVBORw0KGgo=\r\n--B--\r\n"
    )
    .into_bytes()
}
