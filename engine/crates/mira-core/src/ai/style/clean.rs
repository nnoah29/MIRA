//! Nettoyage d'un mail envoyé : on ne garde que ce que l'utilisateur a écrit, sans citation ni signature.

/// Dit si la ligne ouvre la signature (`-- `).
fn is_signature_mark(line: &str) -> bool {
    line.trim_end() == "--"
}

/// Dit si la ligne est l'attaque d'une citation de réponse, en français ou en anglais.
fn is_reply_header(line: &str) -> bool {
    let line = line.trim();
    let french =
        line.starts_with("Le ") && (line.ends_with("a écrit :") || line.ends_with("a écrit:"));
    french || (line.starts_with("On ") && line.ends_with("wrote:"))
}

/// Texte écrit par l'utilisateur : sans citation, signature ni ligne vide, chaque ligne rognée.
pub fn clean_mail(text: &str) -> String {
    let mut kept: Vec<&str> = Vec::new();
    for line in text.lines() {
        if is_signature_mark(line) || is_reply_header(line) {
            break;
        }
        let trimmed = line.trim();
        if !trimmed.starts_with('>') && !trimmed.is_empty() {
            kept.push(trimmed);
        }
    }
    kept.join("\n")
}
