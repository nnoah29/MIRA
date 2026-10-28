-- Libellés système (R4) : marqueurs « à répondre », « en attente », « mentions » posés par le moteur sur les conversations.
-- État par (compte, clé du fil, sorte) : `auto` vient de la détection, `manual` d'un choix de l'utilisateur, valable tant que
-- la conversation n'a pas de message plus récent que `as_of` ; `on_state = 0` mémorise un retrait manuel.
CREATE TABLE thread_system_labels (
    account_id TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    root_key TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('needs', 'waiting', 'mention')),
    on_state INTEGER NOT NULL CHECK (on_state IN (0, 1)),
    source TEXT NOT NULL CHECK (source IN ('auto', 'manual')),
    as_of INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, root_key, kind)
);

-- `labels_dirty` : un message est arrivé dans le fil ; `labels_recheck_at` : instant où le verdict peut changer sans message.
ALTER TABLE threads ADD COLUMN labels_dirty INTEGER NOT NULL DEFAULT 1;
ALTER TABLE threads ADD COLUMN labels_recheck_at INTEGER;
CREATE INDEX threads_labels_pending ON threads(account_id, labels_dirty, labels_recheck_at);
