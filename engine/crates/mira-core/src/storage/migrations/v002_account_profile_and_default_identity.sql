-- Le profil du compte est conservé avec lui ; une seule identité par défaut par compte.
ALTER TABLE accounts ADD COLUMN profile_id TEXT NOT NULL DEFAULT 'default';
CREATE UNIQUE INDEX identities_one_default_per_account ON identities(account_id) WHERE is_default = 1;
