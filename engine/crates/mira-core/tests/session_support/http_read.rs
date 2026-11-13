//! Lecture d'une requête HTTP complète par un faux serveur.

/// Dit si les en-têtes et le corps annoncé sont entièrement arrivés
pub fn request_complete(raw: &[u8]) -> bool {
    let text = String::from_utf8_lossy(raw);
    let Some((head, rest)) = text.split_once("\r\n\r\n") else {
        return false;
    };
    let length = head
        .lines()
        .find_map(|line| {
            line.to_lowercase()
                .strip_prefix("content-length:")
                .map(|v| v.trim().parse::<usize>().unwrap_or(0))
        })
        .unwrap_or(0);
    rest.len() >= length
}
