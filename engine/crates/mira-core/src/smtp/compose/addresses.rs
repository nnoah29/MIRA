//! Adresses du message : validation, en-têtes `From`/`To`/`Cc` et destinataires de l'enveloppe.

use mail_builder::headers::address::Address;
use mira_vocabulary::error::codes::invalid_input;
use mira_vocabulary::{AddressEntry, EmailAddress, ErrorCategory, MiraError};

use super::input::ComposeInput;
use super::sanitize::single_line;

/// Valide une adresse et refuse les caractères qui permettraient d'injecter un en-tête
pub(super) fn checked_address(entry: &AddressEntry) -> Result<EmailAddress, MiraError> {
    let parsed = EmailAddress::parse(&entry.address)?;
    let forbidden = |c: char| c.is_control() || matches!(c, '<' | '>' | '"' | ',' | ';');
    if parsed.as_str().contains(forbidden) {
        return Err(MiraError::new(
            ErrorCategory::InvalidInput,
            invalid_input::ADDRESS,
            "adresse e-mail invalide",
        ));
    }
    Ok(parsed)
}

/// Construit l'adresse d'en-tête (nom d'affichage facultatif) d'une entrée
pub(super) fn header_address(entry: &AddressEntry) -> Result<Address<'static>, MiraError> {
    let address = checked_address(entry)?;
    let name = entry
        .name
        .as_deref()
        .map(single_line)
        .filter(|name| !name.is_empty());
    Ok(Address::new_address(name, address.as_str().to_string()))
}

/// Construit la liste d'adresses d'un en-tête, ou rien si la liste est vide
pub(super) fn header_list(entries: &[AddressEntry]) -> Result<Option<Address<'static>>, MiraError> {
    if entries.is_empty() {
        return Ok(None);
    }
    let items = entries.iter().map(header_address);
    Ok(Some(Address::new_list(items.collect::<Result<_, _>>()?)))
}

/// Destinataires de l'enveloppe : To, Cc puis Bcc, sans doublon, au moins un
pub(super) fn envelope_recipients(input: &ComposeInput) -> Result<Vec<String>, MiraError> {
    let mut recipients: Vec<String> = Vec::new();
    for entry in input.to.iter().chain(&input.cc).chain(&input.bcc) {
        let address = checked_address(entry)?.as_str().to_string();
        if !recipients.contains(&address) {
            recipients.push(address);
        }
    }
    if recipients.is_empty() {
        return Err(MiraError::new(
            ErrorCategory::InvalidInput,
            invalid_input::MISSING_PARAMETER,
            "aucun destinataire",
        ));
    }
    Ok(recipients)
}
