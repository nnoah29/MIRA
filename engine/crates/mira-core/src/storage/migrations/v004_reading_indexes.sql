-- Index des vues calculées (J6) : parcours par date sans tri, tous comptes confondus.
CREATE INDEX messages_by_date ON messages(sort_date DESC, id DESC);
CREATE INDEX messages_unread_by_date ON messages(sort_date DESC, id DESC) WHERE seen = 0;
CREATE INDEX messages_flagged_by_date ON messages(sort_date DESC, id DESC) WHERE flagged = 1;
CREATE INDEX mailboxes_by_account_role ON mailboxes(account_id, role);
