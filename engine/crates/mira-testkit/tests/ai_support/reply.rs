//! Réponse HTTP analysée : statut, en-têtes, corps décodé et état du découpage en morceaux.

/// Réponse reçue ; `complete` est faux si le corps découpé s'arrête avant le morceau final.
pub struct Reply {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub complete: bool,
}

impl Reply {
    /// Valeur du premier en-tête portant ce nom, sans tenir compte de la casse
    pub fn header(&self, name: &str) -> Option<&str> {
        let found = self
            .headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name));
        found.map(|(_, value)| value.as_str())
    }

    /// Corps sous forme de texte
    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

/// Découpe une réponse brute en statut, en-têtes et corps, en décodant le découpage en morceaux
pub fn parse_reply(raw: &[u8]) -> Reply {
    let split = raw.windows(4).position(|w| w == b"\r\n\r\n").unwrap();
    let head = String::from_utf8_lossy(&raw[..split]).into_owned();
    let mut lines = head.split("\r\n");
    let status = lines
        .next()
        .unwrap()
        .split(' ')
        .nth(1)
        .unwrap()
        .parse()
        .unwrap();
    let headers: Vec<(String, String)> = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(k, v)| (k.trim().to_ascii_lowercase(), v.trim().to_string()))
        .collect();
    let chunked = headers
        .iter()
        .any(|(k, v)| k == "transfer-encoding" && v == "chunked");
    let (body, complete) = decode_body(&raw[split + 4..], chunked);
    Reply {
        status,
        headers,
        body,
        complete,
    }
}

/// Décode le corps, découpé en morceaux ou brut, et indique s'il est complet
fn decode_body(raw: &[u8], chunked: bool) -> (Vec<u8>, bool) {
    if !chunked {
        return (raw.to_vec(), true);
    }
    let mut body = Vec::new();
    let mut rest = raw;
    while let Some(eol) = rest.windows(2).position(|w| w == b"\r\n") {
        let size = usize::from_str_radix(std::str::from_utf8(&rest[..eol]).unwrap(), 16).unwrap();
        if size == 0 {
            return (body, true);
        }
        let end = eol + 2 + size;
        if rest.len() < end + 2 {
            return (body, false);
        }
        body.extend_from_slice(&rest[eol + 2..end]);
        rest = &rest[end + 2..];
    }
    (body, false)
}
