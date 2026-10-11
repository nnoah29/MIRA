-- Instant d'expiration du jeton d'accès d'un compte (7.7) ; ce n'est pas un secret.
-- Pas de clé étrangère : les jetons peuvent être rangés avant l'insertion du compte ; un déclencheur nettoie à la suppression.
CREATE TABLE account_token_meta (
    account_id TEXT PRIMARY KEY,
    expires_at INTEGER NOT NULL
);
CREATE TRIGGER account_token_meta_cleanup AFTER DELETE ON accounts
BEGIN
    DELETE FROM account_token_meta WHERE account_id = OLD.id;
END;
