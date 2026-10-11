# J4 — Spécification du testkit (`engine/crates/mira-testkit`)

Faux serveurs **réalistes**, en processus, sur `127.0.0.1` avec un port éphémère (`TcpListener::bind("127.0.0.1:0")`), en clair (pas de TLS : le cœur n'autorise le clair que vers la boucle locale). Ils ne tournent qu'à l'intérieur des tests. Le crate ne dépend d'aucun crate `mira-*`. Dépendances disponibles (workspace) : `tokio`, `base64`, `sha2`, `serde_json`. Toute autre dépendance : demander dans le rapport, ne pas l'ajouter.

Socle existant (ne pas modifier sauf nécessité, et le signaler) : `src/lib.rs` (déclare `credentials`, `imap`, `oauth`, `smtp`), `src/credentials.rs` (`Credentials` : mots de passe et jetons d'accès partagés entre faux serveurs).

Règles de code : celles de `CLAUDE.md` (≤ 30 lignes par fonction, ≤ 6 fonctions par fichier, aucun commentaire dans les corps, une ligne `///` par fonction, pas d'`unwrap`/`expect`/`panic!` dans `src/` : utiliser `Result`, `unwrap_or_else(|p| p.into_inner())` pour les verrous). Les tests du testkit vont dans `tests/<thème>.rs` (premier ligne : `#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]`), avec un client TCP brut écrit dans le test (écrire une commande, lire la réponse).

Arrêt : chaque serveur démarré renvoie un handle ; le `Drop` du handle (ou une méthode `shutdown`) arrête la tâche d'écoute (`tokio::task::JoinHandle::abort`).

---

## Partie A — Faux serveur IMAP (`src/imap/**`, agent « testkit IMAP »)

### API publique (consommée par les tests du moteur)

```rust
pub enum Capability { Idle, Condstore, Qresync, Move, UidPlus, SpecialUse, Enable, Id, Gmail, Literalplus, Esearch }
pub struct CapabilitySet { .. }      // full(), without_qresync(), without_move(), minimal(), gmail(); with(c), without(c), contains(c)
pub struct FakeImapServer { .. }      // handle : port(), address() -> "127.0.0.1:<port>", state() -> FakeMailStore, connection_count(), drop_all_connections(), shutdown()
impl FakeImapServer { pub async fn start(store: FakeMailStore, credentials: Credentials, caps: CapabilitySet) -> std::io::Result<Self> }

pub struct FakeMailStore { .. }       // Arc<Mutex<...>>, clonable ; manipulé par les tests « depuis un autre appareil »
impl FakeMailStore {
    pub fn new() -> Self;
    pub fn create_mailbox(&self, user: &str, name: &str, special_use: Option<&str>);  // "\\Sent", "\\Trash", "\\All", ...
    pub fn deliver(&self, user: &str, mailbox: &str, raw: &[u8], flags: &[&str]) -> u32;   // renvoie l'UID ; réveille les IDLE
    pub fn deliver_gmail(&self, user: &str, labels_mailboxes: &[&str], raw: &[u8], gm_msgid: u64, gm_thrid: u64) -> Vec<u32>; // même message dans plusieurs boîtes
    pub fn set_flags(&self, user: &str, mailbox: &str, uid: u32, flags: &[&str]);   // incrémente MODSEQ
    pub fn expunge(&self, user: &str, mailbox: &str, uid: u32);                       // trace pour VANISHED
    pub fn renumber(&self, user: &str, mailbox: &str);                                 // nouvel UIDVALIDITY, nouveaux UID
    pub fn messages(&self, user: &str, mailbox: &str) -> Vec<FakeMessageView>;         // uid, flags, raw, modseq
    pub fn fail_next(&self, command: &str, response: &str);                            // ex. ("UID MOVE", "NO [OVERQUOTA] quota")
    pub fn command_log(&self) -> Vec<String>;                                          // commandes reçues (sans mots de passe : LOGIN journalisé "LOGIN <user> ***")
}
```

(Le découpage interne en fichiers est libre, ≤ 6 fonctions par fichier : par exemple `imap/store/`, `imap/parser/`, `imap/commands/<commande>.rs`, `imap/server.rs`, `imap/capabilities.rs`.)

### Modèle

Par utilisateur : des boîtes (nom, attribut SPECIAL-USE optionnel, `UIDVALIDITY`, `UIDNEXT`, `HIGHESTMODSEQ`), chaque boîte contenant des entrées (UID, MODSEQ, drapeaux, date interne, identifiant de message partagé). Les messages (source brute, `X-GM-MSGID`, `X-GM-THRID`) sont partagés entre boîtes (modèle Gmail : un message, plusieurs boîtes ; libellés = noms des boîtes qui le contiennent, hors `INBOX` → `\\Inbox`). Hiérarchie avec séparateur `/`. Chaque modification incrémente le `HIGHESTMODSEQ` de la boîte. Les expunges sont mémorisés (UID, MODSEQ) pour `VANISHED (EARLIER)`.

### Commandes à supporter (syntaxe RFC 3501/9051)

- Sans état : `CAPABILITY`, `NOOP`, `LOGOUT`, `ID` (si `Id`), `ENABLE CONDSTORE|QRESYNC` (si `Enable`).
- Authentification : `LOGIN user pass` (chaînes atomiques ou entre guillemets), `AUTHENTICATE PLAIN <b64>` et `AUTHENTICATE XOAUTH2 <b64>` (format `user=<u>\x01auth=Bearer <tok>\x01\x01`, vérifié via `Credentials::check_access_token` ; échec → `NO [AUTHENTICATIONFAILED]`). Capacités annoncées avant authentification : `IMAP4rev1 AUTH=PLAIN AUTH=XOAUTH2` + celles de l'ensemble.
- Boîtes : `LIST "" "*"` (et motifs simples `%`), réponse `* LIST (\HasNoChildren \Sent) "/" "Sent"` ; `STATUS`, `CREATE`, `DELETE`, `RENAME`.
- `SELECT` / `EXAMINE` : `* n EXISTS`, `* OK [UIDVALIDITY v]`, `* OK [UIDNEXT n]`, `* OK [HIGHESTMODSEQ m]` (si Condstore), `* FLAGS (...)`, `OK [READ-WRITE]`. Paramètre `(CONDSTORE)` et `(QRESYNC (uidvalidity modseq [uids]))` : renvoyer `* VANISHED (EARLIER) <uids>` et les `FETCH` modifiés depuis `modseq`.
- `UID FETCH <set> (<items>)` avec items : `UID`, `FLAGS`, `INTERNALDATE`, `RFC822.SIZE`, `ENVELOPE` (facultatif), `BODY.PEEK[HEADER]`, `BODY.PEEK[]`, `BODY[]` (met `\Seen`), `BODY.PEEK[HEADER.FIELDS (...)]`, `MODSEQ` (si Condstore), `X-GM-MSGID`, `X-GM-THRID`, `X-GM-LABELS` (si Gmail), modificateur `(CHANGEDSINCE m)` et `(CHANGEDSINCE m VANISHED)` (si Qresync). Les contenus sont envoyés en littéraux `{n}\r\n<octets>`. Ensembles : `1:*`, `5:10`, `3,7,9`, `*`.
- `UID SEARCH` : `ALL`, `SINCE <dd-Mon-yyyy>`, `BEFORE`, `UID <set>`, `TEXT`, `SUBJECT`, `FROM`, `MODSEQ` ; réponse `* SEARCH ...` (ou `* ESEARCH` si demandé avec `RETURN` et `Esearch`).
- `UID STORE <set> +FLAGS|-FLAGS|FLAGS[.SILENT] (...)` avec `(UNCHANGEDSINCE m)` facultatif ; `+X-GM-LABELS` / `-X-GM-LABELS` (si Gmail : ajoute/retire le message de la boîte du libellé).
- `UID COPY`, `UID MOVE` (si Move ; sinon `BAD`), `UID EXPUNGE` (si UidPlus), `EXPUNGE`, `CLOSE`. Réponses `[COPYUID v src dst]` et `[APPENDUID v uid]` si UidPlus.
- `APPEND mailbox [(flags)] [date] {n}` avec littéral synchronisant (`+ Ready`) et `{n+}` si Literalplus.
- `IDLE` / `DONE` (si Idle) : pendant IDLE, une livraison dans la boîte sélectionnée envoie `* n EXISTS` ; un changement de drapeaux `* n FETCH (FLAGS (...))` ; un expunge `* n EXPUNGE`.
- Commande inconnue ou extension désactivée : `tag BAD ...`. Erreur injectée par `fail_next` : renvoyer la réponse telle quelle.
- `drop_all_connections()` coupe brutalement les sockets (simule une coupure réseau).

### Tests du testkit (au moins)

Login correct et refusé ; XOAUTH2 ; LIST avec SPECIAL-USE ; SELECT avec UIDVALIDITY/HIGHESTMODSEQ ; UID FETCH d'en-têtes et de corps (littéraux) ; STORE incrémente MODSEQ ; QRESYNC renvoie VANISHED et changements ; MOVE absent → BAD ; renumérotation change UIDVALIDITY ; IDLE reçoit EXISTS après `deliver` ; Gmail : un message dans deux boîtes avec le même X-GM-MSGID ; `fail_next`.

---

## Partie B — Faux SMTP et faux serveur OAuth (`src/smtp/**`, `src/oauth/**`, agent « testkit SMTP/OAuth »)

### SMTP

```rust
pub struct FakeSmtpServer { .. }   // start(credentials) ; port() ; received() -> Vec<ReceivedMail { from, recipients, data: Vec<u8> }> ; fail_next(code_and_text) ; drop_after_data_once() ; shutdown()
```

Dialogue RFC 5321 : bannière `220`, `EHLO` (annonce `AUTH PLAIN LOGIN XOAUTH2`, `SIZE`, `8BITMIME`, `PIPELINING` facultatif), `HELO`, `AUTH PLAIN <b64>` / `AUTH LOGIN` (deux étapes) / `AUTH XOAUTH2 <b64>` (vérifiés par `Credentials`), `MAIL FROM:<..>`, `RCPT TO:<..>` (plusieurs), `DATA` → `354`, fin `\r\n.\r\n` (dé-bourrage des points), `250 OK queued as <id>`, `RSET`, `NOOP`, `QUIT`. Pannes : `fail_next("552 5.3.4 Message too big")` s'applique à la prochaine commande qui accepte une erreur (MAIL, RCPT ou fin de DATA) ; `drop_after_data_once()` **enregistre le message comme reçu puis coupe la connexion sans répondre** (simule le plantage « envoyé mais non confirmé », 10.8).

### OAuth2 (code d'autorisation + PKCE S256)

```rust
pub struct FakeOAuthServer { .. }  // start(credentials) ; authorize_url() -> "http://127.0.0.1:<p>/authorize" ; token_url() -> ".../token" ;
                                    // approve(full_authorize_url_with_query, user) -> async : simule le navigateur et l'utilisateur
                                    // revoke_refresh_tokens(user) ; expire_access_tokens(user) ; token_requests() -> nombre de requêtes /token
```

- Mini serveur HTTP/1.1 sur tokio (lecture de la ligne de requête, des en-têtes, du corps selon `Content-Length` ; réponse avec `Content-Length`, `Connection: close`).
- `approve(url, user)` : analyse la requête d'autorisation (`response_type=code`, `client_id`, `redirect_uri`, `state`, `code_challenge`, `code_challenge_method=S256`, `scope`), mémorise un code à usage unique lié au challenge et à l'utilisateur, puis **effectue la redirection** : requête HTTP `GET <redirect_uri>?code=<c>&state=<s>` vers l'écouteur de boucle locale du moteur. Variante `deny(url)` : redirection avec `error=access_denied&state=<s>`.
- `POST /token` (formulaire `application/x-www-form-urlencoded`) :
  - `grant_type=authorization_code&code&redirect_uri&client_id&code_verifier` : vérifie `BASE64URL(SHA256(verifier)) == challenge` et l'usage unique ; réponse JSON `{"access_token","refresh_token","expires_in":3600,"token_type":"Bearer"}` ; le jeton d'accès est enregistré dans `Credentials::add_access_token(user, token)` (ainsi le faux IMAP et le faux SMTP l'acceptent).
  - `grant_type=refresh_token&refresh_token&client_id` : nouveau jeton d'accès (même enregistrement) ; jeton révoqué → `400 {"error":"invalid_grant"}`.
  - Erreurs : `400 {"error":"invalid_request"|"invalid_grant"}`.
- `expire_access_tokens(user)` révoque les jetons d'accès émis pour cet utilisateur (le rafraîchissement reste possible) ; `revoke_refresh_tokens(user)` rend le rafraîchissement impossible (S10).
- Jetons générés de façon déterministe et unique (compteur), jamais écrits dans un journal.

### Tests

SMTP : envoi complet reçu (expéditeur, destinataires, données dé-bourrées) ; AUTH refusé ; `fail_next` ; `drop_after_data_once` (message reçu, connexion coupée sans `250`). OAuth : flux complet avec PKCE (un petit écouteur TCP du test joue le rôle du moteur et reçoit la redirection) ; mauvais verifier refusé ; code réutilisé refusé ; rafraîchissement ; révocation ; le jeton émis est accepté par `Credentials::check_access_token`.
