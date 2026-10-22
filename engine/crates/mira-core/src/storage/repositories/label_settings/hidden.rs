//! Conditions SQL des réglages de libellés : message masqué de la Réception, notifications coupées.

/// Condition « le message `m` porte un libellé dont ce réglage vaut `value` » (serveur : par apparition ; local : par conversation)
pub(crate) fn label_flag_clause(column: &str, value: u8) -> String {
    format!(
        "(EXISTS (SELECT 1 FROM appearances ha JOIN label_settings hs ON hs.kind = 'server' AND hs.label_id = ha.mailbox_id \
         WHERE ha.message_id = m.id AND ha.deleted = 0 AND hs.{column} = {value}) \
         OR EXISTS (SELECT 1 FROM threads ht JOIN thread_local_labels hl ON hl.account_id = ht.account_id AND hl.root_key = ht.root_key \
         JOIN label_settings hs2 ON hs2.kind = 'local' AND hs2.label_id = hl.label_id \
         WHERE ht.id = m.thread_id AND hs2.{column} = {value}))"
    )
}

/// Condition « le message `m` est masqué de la Réception »
pub(crate) fn hidden_from_inbox() -> String {
    label_flag_clause("hide_from_inbox", 1)
}

/// Condition « le message `m` porte un libellé aux notifications coupées »
pub(crate) fn notifications_muted() -> String {
    label_flag_clause("notifications", 0)
}
