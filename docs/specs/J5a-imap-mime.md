# J5a — Client IMAP de haut niveau et analyse MIME

Références : annexe H (client IMAP, analyse MIME), 6.10 (opérations de haut niveau), 10.2, 10.4, 10.14, 11.5, 12.7 (messages mal formés) ; AG-03 (clair seulement vers la boucle locale), AG-06 (tests contre le faux serveur). Testkit : `mira_testkit::imap::{FakeImapServer, FakeMailStore, CapabilitySet}` (voir `docs/specs/J4-testkit.md` partie A et le code réel du testkit).

## 1. Module `mira-core/src/mime/` (encapsule `mail-parser`)

- `ParsedMessage { headers: HeaderSummary, text_body: String, html_body: Option<String>, attachments: Vec<ParsedAttachment>, unsubscribe: Option<Unsubscribe> }`.
- `HeaderSummary { from, to, cc, bcc, reply_to: Vec<AddressEntry>, subject: String, date: Option<Timestamp>, message_id: Option<String>, in_reply_to: Option<String>, references: Vec<String> }` (identifiants sans chevrons).
- `ParsedAttachment { part_index: u32, filename: Option<String>, mime_type: String, size: u64, content_id: Option<String>, inline: bool }`.
- `parse_headers(header_bytes: &[u8]) -> HeaderSummary` (en-têtes seuls, étape 2 de la descente) ; `parse_message(raw: &[u8]) -> ParsedMessage` ; `attachment_bytes(raw: &[u8], part_index: u32) -> Option<Vec<u8>>` ; `preview_text(text: &str) -> String` (≈ 200 caractères, espaces normalisés, coupure sur une frontière de caractère) ; `html_to_text` simple si pas de partie texte (retirer les balises).
- **Tolérance** (12.7) : jamais d'erreur ni de panique sur un message mal formé ; on rend ce qu'on peut (objet vide, corps vide, encodage invalide remplacé). Taille maximale analysée : 50 Mo (au-delà : en-têtes seulement, corps vide, une pièce jointe « trop gros » non décrite) — décision de départ.
- `List-Unsubscribe` / `List-Unsubscribe-Post` (11.6) → `Unsubscribe { https_url, mailto, one_click }`.
- Tests : messages simples, multipart/alternative, pièce jointe, ressource incorporée (`Content-ID`), encodages (quoted-printable, base64, ISO-8859-1, en-têtes RFC 2047 en UTF-8), message tronqué ou sans en-tête, `References` multiples, désabonnement. Un test **proptest** : `parse_message` ne panique jamais sur des octets arbitraires.

## 2. Module `mira-core/src/imap/` (client de haut niveau)

Choix de bibliothèque : `async-imap` 0.12 (runtime tokio) encapsulé ; si elle bloque (QRESYNC, VANISHED, X-GM-*, IDLE avec tokio), écrire un **client maison minimal** (commande étiquetée, lecture des réponses avec littéraux, analyse des réponses non étiquetées utiles). Consigner le choix et la raison dans le rapport (sera reporté en `DECISIONS-AGENT.md`). TLS : `tokio-rustls` + `rustls` avec racines `webpki-roots` (ajouter `webpki-roots` au workspace si nécessaire, le signaler) ; `Plaintext` refusé hors boucle locale (`tls.plaintext_refused`).

API (les noms sont indicatifs, la forme est imposée : des opérations de haut niveau, aucune syntaxe IMAP visible hors du module) :

```rust
pub enum ImapCredential { Password(SecretValue), XOAuth2(SecretValue) }   // SecretValue : ports (J2a)
pub struct ImapLogin { pub user: String, pub credential: ImapCredential }
pub struct ImapClient { .. }
impl ImapClient {
    pub async fn connect(endpoint: &ServerEndpoint, login: &ImapLogin) -> Result<ImapClient, MiraError>;
    pub fn capabilities(&self) -> &[String];                              // annoncées APRÈS authentification
    pub async fn enable_qresync(&mut self) -> Result<bool, MiraError>;
    pub async fn list_mailboxes(&mut self) -> Result<Vec<RawMailbox>, MiraError>;
    pub async fn select(&mut self, mailbox: &str, resync: Option<ResyncFrom>) -> Result<SelectedMailbox, MiraError>;
        // ResyncFrom { uid_validity, highest_modseq } ; SelectedMailbox { uid_validity, uid_next, highest_modseq: Option<u64>, exists, vanished: Vec<u32>, changed: Vec<FetchedFlags> }
    pub async fn uids_since(&mut self, since: Timestamp) -> Result<Vec<u32>, MiraError>;
    pub async fn all_uids(&mut self) -> Result<Vec<u32>, MiraError>;
    pub async fn fetch_headers(&mut self, uids: &[u32], extra: &[String]) -> Result<Vec<FetchedHeaders>, MiraError>;
        // FetchedHeaders { uid, flags: Vec<String>, size, internal_date: Option<Timestamp>, modseq: Option<u64>, header_bytes: Vec<u8>, extra: RawMessageAttributes }
    pub async fn fetch_flags(&mut self, uids: &[u32], changed_since: Option<u64>) -> Result<Vec<FetchedFlags>, MiraError>;
    pub async fn fetch_source(&mut self, uid: u32) -> Result<Option<Vec<u8>>, MiraError>;   // BODY.PEEK[] (ne met pas \Seen)
    pub async fn add_flags / remove_flags(&mut self, uids: &[u32], flags: &[ImapFlag]) -> Result<(), MiraError>;
    pub async fn copy(&mut self, uids: &[u32], to: &str) -> Result<(), MiraError>;
    pub async fn move_messages(&mut self, uids: &[u32], to: &str) -> Result<(), MiraError>;   // MOVE seulement ; le repli copie+supprime est décidé par la synchro
    pub async fn expunge_uids(&mut self, uids: &[u32]) -> Result<(), MiraError>;           // UID EXPUNGE (UIDPLUS)
    pub async fn append(&mut self, mailbox: &str, flags: &[ImapFlag], raw: &[u8]) -> Result<Option<u32>, MiraError>;  // APPENDUID si disponible
    pub async fn create_mailbox / rename_mailbox / delete_mailbox(..) -> Result<(), MiraError>;
    pub async fn add_labels / remove_labels(&mut self, uids: &[u32], labels: &[String]) -> Result<(), MiraError>;  // X-GM-LABELS, utilisé seulement si le provider déclare des libellés
    pub async fn search_text(&mut self, query: &str) -> Result<Vec<u32>, MiraError>;
    pub async fn idle_until_change(&mut self, timeout: Duration) -> Result<IdleOutcome, MiraError>;  // IdleOutcome { Changed, TimedOut }
    pub async fn logout(self) -> Result<(), MiraError>;
}
pub enum ImapFlag { Seen, Flagged, Answered, Deleted, Draft, Forwarded /* $Forwarded */ }
```

- Erreurs → `MiraError` : connexion refusée / injoignable → `network.unreachable` ; coupure → `network.connection_lost` ; délai → `network.timeout` (délais d'attente de 30 s par défaut, injectables) ; `NO [AUTHENTICATIONFAILED]` → `authentication.rejected` (si XOAUTH2 : `authentication.token_expired`, réessayable après rafraîchissement) ; `NO [OVERQUOTA]` → `server.quota_exceeded` ; `BAD` sur une extension → `server.extension_missing` ; autre `NO` → `server.rejected`. **Jamais** de mot de passe ou jeton dans un message d'erreur, un journal (`tracing`) ou un `Debug`.
- Les noms de boîtes sont transmis entre guillemets avec échappement (`"` et `\`) ; UTF-7 modifié non requis en v1 si le serveur accepte UTF-8 (noter la limite).

Tests d'intégration (`engine/crates/mira-core/tests/imap_<thème>.rs`, contre `FakeImapServer` sur 127.0.0.1) : connexion et refus ; XOAUTH2 ; refus du clair vers un hôte non local ; liste des boîtes avec attributs ; SELECT (marqueurs) ; `uids_since` ; en-têtes + attributs supplémentaires Gmail ; source complète ; drapeaux (ajout/retrait) ; QRESYNC (`vanished`, `changed`) ; MOVE ; COPY + EXPUNGE ; APPEND avec APPENDUID ; IDLE réveillé par une livraison ; coupure de connexion → `network.connection_lost` ; jeu de capacités minimal (MOVE absent → `server.extension_missing`).
