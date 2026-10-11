# J3 — Providers, registre et capacités

Références : ch. 6 (tout), D-05, D-06, D-07, D-08, 7.8, 7.9, 9.9, 10.14 ; invariants 1, 2, 3. Interface : `mira_vocabulary::Provider` (voir `docs/specs/J1-vocabulaire.md`, section « Interface provider »).

## 1. Les quatre crates providers (purs : aucune I/O, aucun état, structure unitaire `pub struct GmailProvider;` etc.)

Chaque crate `mira-provider-<x>` : `src/lib.rs` (réexport), `src/descriptor.rs` (données), `src/provider.rs` (`impl Provider`), au besoin `src/roles.rs`, `src/translate.rs` ; tests par tables dans `tests/<thème>.rs` (6.11). Dépendance unique : `mira-vocabulary`. Toutes les valeurs ci-dessous viennent du §6.7 et sont **à vérifier** (doc `///` ou `//!` qui le dit, et liste dans `docs/ARCHITECTURE-IMPLEMENTEE.md` plus tard).

| Provider | id | Domaines | IMAP | SMTP | Auth (ordre) | Limites | Particularités |
|---|---|---|---|---|---|---|---|
| Gmail | `gmail` | gmail.com, googlemail.com | imap.gmail.com:993 TLS | smtp.gmail.com:465 TLS | OAuth2 (authorize `https://accounts.google.com/o/oauth2/v2/auth`, token `https://oauth2.googleapis.com/token`, scope `https://mail.google.com/`), puis AppPassword | 5 connexions, 120 s | `mailbox_model: Labels`, `archive: RemoveFromInbox`, `server_saves_sent_copy: true`, `extra_fetch_attributes: ["X-GM-MSGID","X-GM-THRID","X-GM-LABELS"]` ; rôles : `[Gmail]/Sent Mail`→Sent, `[Gmail]/Drafts`→Drafts, `[Gmail]/Trash`→Trash, `[Gmail]/Spam`→Junk, `[Gmail]/All Mail`→All, `[Gmail]/Starred`→Flagged, ainsi que leurs variantes `[Google Mail]/…` ; `translate_attributes` : `X-GM-MSGID` → `stable_id`, `X-GM-THRID` → `thread_hint`, `X-GM-LABELS` (liste parenthésée, chaînes entre guillemets, libellés système `\\Inbox`, `\\Sent`…) → `labels` |
| Outlook | `outlook` | outlook.com, hotmail.com, live.com, msn.com, + variantes régionales (hotmail.fr, outlook.fr, live.fr, hotmail.co.uk, outlook.de, hotmail.de, hotmail.it, outlook.es, hotmail.es, live.co.uk, live.de, live.it, outlook.be, outlook.jp, hotmail.be, live.ca, hotmail.ca, live.com.au, outlook.com.au) | outlook.office365.com:993 TLS | smtp.office365.com:587 STARTTLS | OAuth2 seulement (authorize `https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize`, token `https://login.microsoftonline.com/consumers/oauth2/v2.0/token`, scopes `https://outlook.office.com/IMAP.AccessAsUser.All`, `https://outlook.office.com/SMTP.Send`, `offline_access`) | 4 connexions, 120 s | dossiers ; `server_saves_sent_copy: true` (à vérifier) ; rôles : `Sent Items`→Sent, `Deleted Items`→Trash, `Junk Email`/`Junk`→Junk, `Archive`→Archive, `Drafts`→Drafts |
| Yahoo | `yahoo` | yahoo.com, ymail.com, rocketmail.com, + variantes (yahoo.fr, yahoo.co.uk, yahoo.de, yahoo.it, yahoo.es, yahoo.ca, yahoo.com.au, yahoo.co.jp, yahoo.com.br) | imap.mail.yahoo.com:993 TLS | smtp.mail.yahoo.com:465 TLS | AppPassword | 3 connexions, 300 s | dossiers ; `server_saves_sent_copy: false` (à vérifier) ; rôles : `Sent`, `Draft`→Drafts, `Trash`, `Bulk Mail`/`Bulk`→Junk, `Archive` |
| Autre | `generic` | aucun (repli) | `imap.<domaine>`:993 TLS | `smtp.<domaine>`:465 TLS | Password, AppPassword | défauts prudents (3, 300 s) | aucune particularité (`ProviderQuirks::default()`, `mailbox_role_hint` → `None`) ; `handles_address` → **toujours vrai** |

`handles_address` : comparaison du domaine (déjà en minuscules dans `EmailAddress`) avec la liste exacte ; un sous-domaine (`foo.gmail.com`) **n'est pas** reconnu. `mailbox_role_hint` ne répond que pour les noms propres au service ; sinon `None` (le cœur applique la règle générale).

## 2. Registre (`mira-core/src/registry/`)

- `ProviderRegistry { providers: Vec<Arc<dyn Provider>>, fallback: Arc<dyn Provider> }` : `new(fallback)`, `register(provider)`, `find(&ProviderId) -> Option<Arc<dyn Provider>>` (le repli est aussi trouvable par son id), `detect(&EmailAddress) -> Arc<dyn Provider>` (premier qui répond oui dans l'ordre d'enregistrement, sinon le repli — 6.6), `list() -> Vec<ProviderDescriptor>` (y compris le repli, en dernier).
- Provider introuvable pour un compte (7.8) : `find` renvoie `None` ; l'appelant (comptes) mettra le compte en `AccountState::Error { code: "not_found.provider" }` sans toucher à ses données. Fournir la fonction `registry::resolve_account_provider(&registry, &ProviderId) -> Result<Arc<dyn Provider>, MiraError>` qui renvoie l'erreur `not_found.provider`.

## 3. Rôle des dossiers (`mira-core/src/mailboxes/role.rs` ou `registry/roles.rs`)

`infer_mailbox_role(provider: &dyn Provider, raw: &RawMailbox) -> MailboxRole` : (1) attribut SPECIAL-USE (`\\Sent`, `\\Drafts`, `\\Trash`, `\\Junk`, `\\Archive`, `\\All`, `\\Flagged`), (2) nom `INBOX` (insensible à la casse) → Inbox, (3) indice du provider, (4) noms usuels en plusieurs langues (dernier segment du chemin, insensible à la casse) : Sent/Sent Items/Sent Messages/Envoyés/Éléments envoyés/Gesendet/Inviati/Enviados → Sent ; Drafts/Brouillons/Entwürfe/Bozze/Borradores → Drafts ; Trash/Deleted/Deleted Items/Deleted Messages/Corbeille/Papierkorb/Cestino/Papelera → Trash ; Junk/Spam/Junk E-mail/Courrier indésirable/Indésirables → Junk ; Archive/Archives/Archiv → Archive ; (5) sinon Other. **Ordre important** : SPECIAL-USE gagne toujours (10.14).

## 4. Capacités effectives (`mira-core/src/capabilities/`)

- `ServerCapability` (énumération du moteur, indépendante d'IMAP brut) : `Idle`, `Notify`, `Condstore`, `Qresync`, `Move`, `UidPlus`, `SpecialUse`, `ObjectId`, `Replace`, `Esearch`, `MultiSearch`, `Compress`, `LiteralPlus`, `Utf8Accept`, `Id`, `Enable`.
- `parse_capabilities(raw: &[String]) -> BTreeSet<ServerCapability>` (insensible à la casse, `COMPRESS=DEFLATE` → Compress, `UTF8=ACCEPT` → Utf8Accept, inconnus ignorés).
- `EffectiveCapabilities::compute(announced: &BTreeSet<ServerCapability>, quirks: &ProviderQuirks) -> EffectiveCapabilities` : annonce du serveur moins les extensions déclarées peu fiables par le provider (`unreliable_extensions`, noms IMAP comparés sans casse) ; `has(cap)`. (D-08 : le serveur est la vérité, le provider ne fournit que des exceptions.)
- `SyncStrategyChoice` : `choose_change_detection(&EffectiveCapabilities) -> ChangeDetection { Qresync | Condstore | FullCompare }`, `choose_move(..) -> MoveMethod { Move | CopyThenDelete }`, `choose_realtime(..) -> Realtime { Idle | Polling }`, `choose_expunge(..) -> ExpungeMethod { UidExpunge | NoPurge }` (prudence de 10.14 : sans UIDPLUS on ne purge pas). Chaque choix de repli est journalisé par l'appelant (pas ici).

## 5. Assemblage (`mira-assembly`)

`pub fn build_registry() -> ProviderRegistry` : repli = generic ; enregistre gmail, outlook, yahoo (une ligne chacun, 5.4). `pub use mira_core::...` des éléments du contrat dont le CLI aura besoin (réexport de `mira_core::api` quand il existera ; pour l'instant `registry` et `mira_vocabulary` réexportés).

## 6. CLI minimal (phase 1 de la feuille de route)

`mira provider list` et `mira provider detect <adresse>` (sortie humaine par défaut, `--json` = enveloppe `{"contract_version":{"major":1,"minor":0},"ok":true,"data":…}` ; en erreur `{"…","ok":false,"error":<MiraError>}` sur stdout avec `--json`, message lisible sur stderr sinon ; code de sortie 0 / 10 (entrée invalide) / 84). Dépendances CLI : `clap` (derive), `serde_json`, `anyhow` interdit dans le chemin du contrat (seulement pour `main`). Code de sortie par catégorie : `invalid_input` 10, `not_found` 11, `incompatible_state` 12, `network` 13, `tls` 14, `authentication` 15, `server` 16, `local_storage` 17, `secrets` 18, `cancelled` 19, `internal` 84.
Le CLI expose `pub fn run(args: Vec<String>, deps: CliDeps) -> CliOutcome` testable en processus (stdout/stderr capturés dans des `Vec<u8>`), `main` n'étant qu'un appel à `run` avec les vraies dépendances.

## 7. Tests exigés

- Providers : tables de reconnaissance (y compris majuscules → l'adresse est normalisée, sous-domaine refusé, domaines régionaux), serveurs, rôles (noms réels), traduction Gmail (`X-GM-LABELS ("\\Inbox" "Travail" "Factures")`), limites.
- Cœur : registre avec **faux providers définis dans les tests du cœur** (`FakeLabelsProvider`, `FakeFoldersProvider`) : ordre de détection, repli, provider introuvable ; `infer_mailbox_role` (table) ; capacités : exception du provider retire une extension annoncée ; choix de stratégie pour les profils complet / sans QRESYNC / sans MOVE / minimal.
- Assemblage : `invariant_1_core_names_no_provider` n'est pas un test Rust (c'est le garde-fou A6) ; un test d'assemblage vérifie que `detect("x@gmail.com")` donne `gmail`, `x@inconnu.org` donne `generic`, et que `list()` contient les 4.
- CLI : `scenario_cli_provider_detect_json` (sortie JSON versionnée), code de sortie 10 sur adresse invalide.
