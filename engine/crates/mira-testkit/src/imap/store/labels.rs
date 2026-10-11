//! Libellés Gmail : noms des boîtes qui contiennent un même message.

use super::data::UserData;

impl UserData {
    /// Libellés d'un message : noms des boîtes qui le contiennent, INBOX devenant `\Inbox`
    pub(crate) fn labels_of(&self, gm_msgid: u64) -> Vec<String> {
        self.mailboxes
            .iter()
            .filter(|(_, mailbox)| mailbox.special_use.as_deref() != Some("\\All"))
            .filter(|(_, mailbox)| {
                mailbox
                    .entries
                    .iter()
                    .any(|entry| entry.message.gm_msgid == gm_msgid)
            })
            .map(|(name, _)| match name.as_str() {
                "INBOX" => "\\Inbox".to_string(),
                other => other.to_string(),
            })
            .collect()
    }

    /// Nom de boîte désigné par un libellé (`\Inbox` ou nom)
    pub(crate) fn mailbox_for_label(&self, label: &str) -> String {
        if label.eq_ignore_ascii_case("\\Inbox") {
            return "INBOX".to_string();
        }
        self.mailboxes
            .iter()
            .find(|(_, mailbox)| mailbox.special_use.as_deref() == Some(label))
            .map_or_else(|| label.to_string(), |(name, _)| name.clone())
    }
}
