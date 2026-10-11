-- MIRA schéma v1 : trois familles de données (D-18). Aucun secret n'est stocké ici (P9).

-- Famille configuration (irremplaçable)
CREATE TABLE accounts (
    id TEXT PRIMARY KEY,
    address TEXT NOT NULL,
    display_name TEXT NOT NULL,
    provider_id TEXT NOT NULL,
    auth_kind TEXT NOT NULL,
    servers_json TEXT,
    sync_json TEXT NOT NULL,
    state_json TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    deleting INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE identities (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    address TEXT NOT NULL,
    display_name TEXT NOT NULL,
    signature TEXT,
    is_default INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX identities_by_account ON identities(account_id);

CREATE TABLE remote_content_allowances (
    kind TEXT NOT NULL CHECK (kind IN ('message', 'sender')),
    value TEXT NOT NULL,
    PRIMARY KEY (kind, value)
);

-- Famille miroir du serveur (jetable et reconstruisible)
CREATE TABLE mailboxes (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    display_name TEXT NOT NULL,
    delimiter TEXT,
    parent_id TEXT,
    role TEXT NOT NULL,
    sync_enabled INTEGER NOT NULL DEFAULT 1,
    uid_validity INTEGER,
    uid_next INTEGER,
    highest_modseq INTEGER,
    total INTEGER NOT NULL DEFAULT 0,
    unread INTEGER NOT NULL DEFAULT 0,
    UNIQUE (account_id, name)
);

CREATE TABLE threads (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    subject TEXT NOT NULL,
    root_key TEXT NOT NULL,
    UNIQUE (account_id, root_key)
);

CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    thread_id TEXT,
    from_json TEXT NOT NULL,
    to_json TEXT NOT NULL,
    cc_json TEXT NOT NULL,
    bcc_json TEXT NOT NULL,
    reply_to_json TEXT NOT NULL,
    subject TEXT NOT NULL,
    date INTEGER,
    internal_date INTEGER,
    message_id_header TEXT,
    in_reply_to TEXT,
    references_json TEXT NOT NULL,
    seen INTEGER NOT NULL DEFAULT 0,
    flagged INTEGER NOT NULL DEFAULT 0,
    answered INTEGER NOT NULL DEFAULT 0,
    forwarded INTEGER NOT NULL DEFAULT 0,
    size INTEGER NOT NULL DEFAULT 0,
    preview TEXT NOT NULL DEFAULT '',
    has_attachments INTEGER NOT NULL DEFAULT 0,
    unsubscribe_json TEXT,
    content_state TEXT NOT NULL,
    sort_date INTEGER NOT NULL
);
CREATE INDEX messages_by_account_date ON messages(account_id, sort_date DESC, id DESC);
CREATE INDEX messages_by_thread ON messages(thread_id);
CREATE INDEX messages_by_message_id_header ON messages(account_id, message_id_header);

CREATE TABLE appearances (
    message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    mailbox_id TEXT NOT NULL REFERENCES mailboxes(id) ON DELETE CASCADE,
    uid INTEGER NOT NULL,
    deleted INTEGER NOT NULL DEFAULT 0,
    needs_revalidation INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (message_id, mailbox_id),
    UNIQUE (mailbox_id, uid)
);
CREATE INDEX appearances_by_mailbox ON appearances(mailbox_id, message_id);

CREATE TABLE server_ids (
    account_id TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('stable', 'uid')),
    value TEXT NOT NULL,
    message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    PRIMARY KEY (account_id, kind, value)
);
CREATE INDEX server_ids_by_message ON server_ids(message_id);

CREATE TABLE raw_sources (
    message_id TEXT PRIMARY KEY REFERENCES messages(id) ON DELETE CASCADE,
    compressed BLOB NOT NULL,
    raw_size INTEGER NOT NULL
);

CREATE TABLE parsed_cache (
    message_id TEXT PRIMARY KEY REFERENCES messages(id) ON DELETE CASCADE,
    text_body TEXT NOT NULL,
    html_body TEXT
);

CREATE TABLE attachments (
    id TEXT PRIMARY KEY,
    message_id TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    part_index INTEGER NOT NULL,
    filename TEXT,
    mime_type TEXT NOT NULL,
    size INTEGER NOT NULL,
    content_id TEXT,
    inline INTEGER NOT NULL DEFAULT 0,
    cache_path TEXT
);
CREATE INDEX attachments_by_message ON attachments(message_id);

CREATE VIRTUAL TABLE messages_fts USING fts5(
    message_id UNINDEXED,
    subject,
    addresses,
    body,
    tokenize = 'unicode61 remove_diacritics 2'
);

-- Famille intentions locales (jamais jetée)
CREATE TABLE drafts (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    identity_id TEXT NOT NULL,
    to_json TEXT NOT NULL,
    cc_json TEXT NOT NULL,
    bcc_json TEXT NOT NULL,
    subject TEXT NOT NULL,
    body_text TEXT NOT NULL,
    body_html TEXT,
    in_reply_to TEXT,
    state_json TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE draft_attachments (
    draft_id TEXT NOT NULL REFERENCES drafts(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    filename TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    data BLOB NOT NULL,
    PRIMARY KEY (draft_id, position)
);

CREATE TABLE actions (
    seq INTEGER PRIMARY KEY AUTOINCREMENT,
    id TEXT NOT NULL UNIQUE,
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    kind_json TEXT NOT NULL,
    targets_json TEXT NOT NULL,
    server_targets_json TEXT NOT NULL,
    idempotency_key TEXT NOT NULL,
    state TEXT NOT NULL,
    attempts INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    created_at INTEGER NOT NULL,
    not_before INTEGER NOT NULL,
    undo_json TEXT,
    UNIQUE (account_id, idempotency_key)
);
CREATE INDEX actions_by_account_state ON actions(account_id, state, seq);

-- Événements numérotés (12.5)
CREATE TABLE events (
    seq INTEGER PRIMARY KEY AUTOINCREMENT,
    at INTEGER NOT NULL,
    account_id TEXT,
    category TEXT NOT NULL,
    kind_json TEXT NOT NULL
);
