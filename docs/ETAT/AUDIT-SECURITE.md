# Audit qualité et sécurité — HEAD f91f2d4

Chemins relatifs à `engine/crates/mira-core/src` (C). Vérifier avant de corriger ; test de non-régression d'abord.

Sain (vérifié) : aucun `unwrap/expect/panic!/unsafe` hors tests ; SQL paramétré, FTS échappé ; TLS strict (webpki, SNI, tampon jeté après STARTTLS, clair refusé hors boucle locale) ; OAuth loopback PKCE S256, `state`, page statique CSP ; `Debug` masqué des secrets, pas de secret dans journaux/erreurs/exports ; profil 0700, écriture atomique des pièces jointes.

## Élevé
1. = revue #1 (HTML quadratique sous verrou DB). Aussi `C/api/handlers/message/safe_html.rs:35-44` : assainir HORS `db.read` (`spawn_blocking`), idem trousseau (appels D-Bus synchrones) et `integrity_check`.
2. = revue #10 + : `C/html/remote.rs:29-35`, `sanitize.rs:56` — tabulation/saut de ligne/C0 dans l'URL, attribut `background`, `\\host`. Liste blanche (`cid:`, `data:image/*` hors SVG, seulement dans `img src`), normaliser (retirer tab/LF, C0 et espaces aux extrémités, `\`→`/`), `UrlRelative::Deny`. Constat 19 : `data:` interdit dans `<a href>`.
3. « Message poison » : `C/imap/protocol/reader.rs:58-63` (`timeout` de 30 s sur la ligne logique ET tous ses littéraux ; `imap/types/login.rs:7`), `C/sync/download/prefetch.rs:68-69` (`fetch_one(...)?` fatal), `account.rs:24-36`. Corriger : délai d'inactivité réarmé à chaque bloc ; dans `prefetch_bodies` journaliser et ignorer l'échec d'un message (ne pas réessayer indéfiniment) ; plafonner la taille préchargée via `RFC822.SIZE`.
4. = revue #3 + : `ETIMEDOUT` après DATA classé passager → doublon. Pour une action `Send`, toute erreur de transport après la connexion (dont `network.timeout`) → `settle_interrupted_send` (incertain) ; passager seulement ce qui est prouvé avant DATA (`UNREACHABLE`, 4xx). `C/actions/replay/settle.rs:21`, `classify.rs`, `park.rs:62-63`, `smtp/errors.rs:25-31`.

## Moyen
5. IMAP mémoire non bornée : `C/imap/protocol/reader.rs:17-21,36-54`, `command.rs:11-20`, `literal_command.rs:13` → plafond de ligne physique 1 Mo, cumul 64 Mo par ligne logique et par réponse, allocation progressive (`take(size).read_to_end`), test avec serveur scripté.
6. = revue #6 : lectures explicites hors budget (`C/fetching/access.rs:36-50`, `download.rs:14`, `server_search.rs:49`) ; borner aussi le nombre de tâches `run_in_background` par compte.
8. Noms de pièces jointes non nettoyés (`C/mime/attachments.rs:33`) : dernier composant, retirer `/ \ NUL`, contrôles, bidi, refuser `.`/`..`, 255 octets ; idem `mime_type`.
9. Table `events` jamais purgée (`C/storage/repositories/events/prune.rs:10`, aucun appelant) : appeler à l'ouverture et après chaque passe ; rétention à consigner (AG).

## Bas
10. `C/maintenance/export_file.rs:24-37` : `create_new(true)`, `O_NOFOLLOW`, suffixe aléatoire.
11. `C/drafts/attachments/local_file.rs:23-36` : ouvrir une fois, `file.metadata()`, `take(MAX+1)`.
12. `C/auth/flow/redirect.rs:63-90` : `state` faux → 400 et continuer d'écouter ; une tâche par connexion avec délai ; comparaison en temps constant.
13. Fuites : `C/auth/flow_table` (TTL, plafond), `C/operations/registry.rs:43-50` (purge).
14. `C/auth/tokens/http.rs:32-43` : lire par `chunk()` borné à 1 Mo.
15. `C/auth/loopback.rs:30-39` : `reqwest::Url::parse` ; un seul `is_loopback_host`.
16. Dates serveur : `C/imap/parse/dates.rs:39-41`, `civil.rs:4-12` → année 1900–9999, champs bornés, `checked_*`.
17. Secrets non effacés de la mémoire (`SecretValue(String)`, `imap/connection/authenticate.rs:17,22`, `smtp/transport.rs:24`) : `zeroize` (à consigner) ou limiter les copies.
18. `mira-cli/src/argument_context.rs:23-29` `read_secret` : propager l'erreur de lecture, refuser > 4096 octets et non-UTF-8.
20. Dépendances inutilisées : `thiserror` dans mira-core, feature `json` de reqwest, `async-imap`, `futures`, `anyhow` du workspace si non utilisés.
