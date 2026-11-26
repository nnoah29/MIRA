# Constats de sécurité MIRA — en clair

> **Pour reprendre le chantier : lire `RESTE-A-FAIRE.md`** (ce qui reste, dans l'ordre, les reports et leurs déclencheurs, les décisions prises).

Chaque constat dit : **le problème**, **le contexte** (pourquoi ça compte), **les plateformes touchées**, **le correctif**. Ils sont classés du plus grave au moins grave. La checklist technique (`AUDIT-SECURITE-CHECKLIST.md`) en est le complément.

**Ce qui a été vérifié** : lecture du code du moteur, du démon, de l'application Flutter et du manifeste Android. **Ce qui ne l'a pas été** : aucun test en conditions réelles. Les points marqués « à confirmer par un test » sont une hypothèse fondée sur le code, pas une faille démontrée.

**Vue d'ensemble** : la base est sérieuse (TLS partout, secrets dans le trousseau, jeton du démon comparé en temps constant, images distantes bloquées, IA qui *propose* au lieu d'agir, données sensibles masquées avant envoi à l'IA). Les failles ci-dessous sont celles qui restent.

---

## 1. Le corps des mails s'affiche presque sans filtre dans la vue web — gravité HAUTE

**Le problème.** Pour afficher un mail HTML, l'application n'utilise pas la version nettoyée par le moteur. Elle charge le HTML d'origine (décision AG-47, `mira-host/src/original_html.rs`) dans une vue web où JavaScript est activé (`web_mail_body.dart:141`). La seule barrière est une règle de sécurité (CSP) insérée en tête de page, qui interdit tout chargement et tout script.

**Pourquoi c'est un risque.** Un mail est écrit par un inconnu. Si la règle CSP est contournée une seule fois (faille du moteur web, cas particulier non prévu), le script de l'expéditeur s'exécute dans l'application. C'est la faille classique des clients mail : un seul verrou, aucune seconde ligne de défense.

**Trois trous précis déjà visibles dans le code :**

- **a) Les redirections automatiques ne sont pas bloquées.** La règle CSP n'interdit ni `<meta http-equiv="refresh">`, ni l'envoi de formulaires (`form-action`), ni `<base>`. Quand la page tente une navigation, `_onNavigate` (`web_mail_body.dart:~213`) appelle `openLink`, qui ouvre le navigateur du système vers n'importe quelle adresse http(s). Un mail pourrait donc ouvrir un site (suivi, hameçonnage) **sans que l'utilisateur clique**. *À confirmer par un test sur chaque plateforme.*
- **b) Les adresses `data:` sont autorisées sans condition.** Le même `_onNavigate` laisse passer toute navigation vers `about:` ou `data:`. Un lien `data:text/html,…` dans un mail ouvre alors une page de l'expéditeur **sans la règle CSP**, avec JavaScript actif. Elle peut contacter l'extérieur librement (révèle l'adresse IP, confirme la lecture, imite un écran de connexion).
- **c) JavaScript est actif alors que le contenu est hostile.** Il ne sert qu'à mesurer la hauteur de la page et à relayer la molette, et cela donne à une éventuelle exécution de script un canal vers l'application (`MiraMail`).

**Statut (2026-11-18).** Trous **a** et **b** corrigés dans `app/lib/engine/web_document.dart` et `web_navigation.dart` (tests : `web_document_hardening_test.dart`, `web_navigation_test.dart`) : règle CSP complétée, `meta refresh`, `http-equiv` inconnus et `<base>` retirés avant chargement, `about:`/`data:` refusés une fois le document chargé. **Trou c corrigé (2026-11-22)** : le HTML d'origine passe par `sanitize_original_html` (`mira-core/src/html/original.rs`, appelé par `mira-host/src/original_html.rs`) avant d'atteindre la vue web. Les scripts, cadres, `object`, `embed`, formulaires, `<base>`, `<meta>` et gestionnaires `on…` sont retirés ; styles, classes et attributs de mise en page des tableaux sont gardés. Les images (y compris distantes) restent : la règle CSP de la vue décide du chargement. Tests : `html_original_sanitize.rs`, `original_html_sanitized.rs` ; contrat `contract_direct` vert. **Navigation automatique fermée (2026-11-26, SEC-04)** : `NavigationRequest` ne dit pas si un clic en est l'origine ; le script de page relaie donc lui-même les clics de confiance sur `a[href]`/`area[href]` (`event.isTrusted`, `preventDefault`) et `_onNavigate` bloque toute navigation demandée par la page (`automaticNavigationVerdict`, `clickedLink` ; `web_click_origin_test.dart`). Vérification réelle ajoutée à `VERIFICATIONS-REELLES.md`. **Reste** la vérification réelle sur chaque plateforme (WebKitGTK, WebView, WebView2), et JavaScript reste actif dans la vue pour la mesure de hauteur.

**Plateformes.** Linux (WebKitGTK), Android (WebView), Windows (WebView2) ; macOS le sera aussi quand il arrivera. Le rendu natif en widgets (sans vue web) n'est pas concerné.

**Correctif.**
1. Compléter la règle CSP : `form-action 'none'; base-uri 'none'; frame-src 'none'; object-src 'none'`.
2. Dans `_onNavigate`, n'accepter que le chargement initial du document, et refuser tout `data:` ensuite. N'ouvrir le navigateur que pour un lien cliqué par l'utilisateur (refuser les navigations d'une page qui n'est pas en cours d'interaction).
3. Avant chargement, passer le HTML d'origine dans un nettoyage léger côté moteur : retirer `script`, `iframe`, `object`, `embed`, `form`, `base`, `meta refresh`, attributs `on…`, en gardant les styles et la structure. Cela garde le rendu fidèle tout en ajoutant une vraie seconde ligne de défense.
4. Écrire des tests avec des mails piégés (refresh, formulaire, lien `data:`, `<base>`).

---

## 2. Le désabonnement « en un clic » fait confiance à l'expéditeur — gravité MOYENNE (point 2 reporté)

**Le problème.** Si un mail annonce un désabonnement en un clic, MIRA envoie directement une requête vers l'adresse fournie par le mail (`state/unsubscribe.dart`). Le moteur exige du HTTPS (`mime/unsubscribe.rs:30`), mais rien d'autre n'est contrôlé.

**Pourquoi c'est un risque.** Un mail falsifié (usurpation d'un vrai expéditeur) peut fournir n'importe quelle adresse HTTPS, y compris une machine du réseau local (`https://192.168.x.x/…`) ou un site qui, en voyant la requête, sait que votre adresse est active. L'appel part de l'ordinateur de l'utilisateur, donc depuis son réseau privé.

**Statut (2026-11-22).** Points 1 et 3 corrigés. Le moteur (`mime/unsubscribe_host.rs`) n'expose plus une adresse `https` visant une machine locale ou privée (boucle, `10.x`, `172.16–31.x`, `192.168.x`, `169.254.x`, `100.64/10`, IPv6 locales, noms sans point, `.local`, identifiants dans l'adresse) : le mailto reste proposé, sans « un clic ». Côté application (`state/unsubscribe.dart`), le nom est résolu et chaque adresse obtenue contrôlée (contre un nom public qui pointe en privé), les redirections ne sont pas suivies, la réponse est limitée à 64 Ko et 10 s. Tests : `mime_unsubscribe_private_hosts.rs`, `unsubscribe_one_click_safety_test.dart`. **Point 2 reporté (décision du 2026-11-26)** (exiger un DKIM valide) : le moteur ne vérifie pas encore DKIM. Raison : gros chantier (vérification de signature, clés DNS, mails non signés) pour un risque déjà réduit par les points 1 et 3 ; il reste qu'un faux mail peut faire contacter un site public, ce qui révèle qu'une adresse est active (confidentialité, faible). Déclencheur de reprise : le jour où MIRA vérifie DKIM pour une autre raison (badge d'authentification, alerte constat 8). **Mineurs corrigés (2026-11-26, SEC-04)** : plages oubliées ajoutées des deux côtés (`mime/unsubscribe_ranges.rs`, `isPublicAddress`) — `255.255.255.255`, `240.0.0.0/4`, `198.18.0.0/15`, `192.0.0.0/24`, `fec0::/10`, `::a.b.c.d` (tout `::/96`), NAT64 `64:ff9b::/96` vers IPv4 privée, `ff00::/8` ; tests `mime_unsubscribe_reserved_v4/v6.rs`, `unsubscribe_reserved_ranges_test.dart`. **Rebinding corrigé** : `HttpClient.connectionFactory` se connecte à l'adresse déjà contrôlée, TLS négocié sur le nom (`unsubscribe_dns_rebinding_test.dart`, certificat de test local).

**Plateformes.** Toutes.

**Correctif.**
1. Refuser les adresses privées ou locales (boucle locale, `10.x`, `172.16–31.x`, `192.168.x`, `169.254.x`, noms sans point).
2. Ne proposer le désabonnement automatique que si le mail est authentifié (DKIM valide, comme l'exige la RFC 8058) ; sinon, ouvrir la page dans le navigateur après confirmation.
3. Ne pas suivre les redirections et limiter la taille de la réponse.

---

## 3. Aucune surveillance des failles connues dans les dépendances — gravité MOYENNE

**Le problème.** La CI vérifie les licences et les sources (`ci.sh:42` : `cargo deny check bans licenses sources`), mais **exclut volontairement les avis de sécurité**. Côté Flutter (`pubspec.lock`), aucun contrôle n'existe.

**Pourquoi c'est un risque.** MIRA embarque des composants qui lisent des données hostiles : `ammonia` (HTML), `mail_parser` / `mail_builder` (mails), `zip` / `tar` / `flate2` (archives), `rustls` (TLS), `reqwest`. Une faille publiée dans l'un d'eux resterait inconnue tant que personne ne la cherche.

**Attention (2026-11-26) : ces mises à jour n'avaient été faites que dans `engine/Cargo.lock` ; le pont livré a son propre verrou, corrigé depuis (constat 12).** **Statut (2026-11-22).** Premier passage de `cargo deny check advisories` : **plus de 25 avis réels**, dont `ammonia` (4 avis, dont XSS par SVG/MathML : c'est l'assainisseur du HTML des mails), `lettre` (injection de commandes SMTP, usurpation de réponses STARTTLS, mauvais destinataire), `rustls`/`rustls-webpki` (certificats refusés à tort acceptés, rétrogradation d'échange de clés), `url`, `zstd`, `tokio`, `rusqlite`/SQLite (CVE). **Tous corrigés** par mise à jour du verrou (`ammonia` 4.2.5, `lettre` 0.11.24, `rustls` 0.23.46, `rustls-webpki` 0.103.17, `tokio` 1.54.1, `url` 2.5.9, `zstd` 0.14.4, `webpki-roots` 1.0.12) et montée de `rusqlite` 0.40 → 0.41 (SQLite embarqué corrigé), sans changement de code ; tests ciblés HTML, SMTP et stockage verts. **Restent 4 avis « non maintenu »** (`chrono`, `zbus_names`, `zvariant`, `zvariant_derive`), sans correctif possible, inscrits avec justification dans `engine/deny.toml`. Ajoutés : `.github/workflows/securite.yml` (hebdomadaire, à la main ou sur une PR vers `main` : avis RustSec et OSV sur `pubspec.lock` ; `release.yml` ne le déclenche pas) et `.github/dependabot.yml`. **SBOM (2026-11-26)** : `tools/generate-sbom` produit un CycloneDX 1.5 (Rust par `cargo metadata`, Dart par `pubspec.lock`, sans réseau ni outil externe), joint à chaque release par `release.yml` et `packaging/release.sh`, couvert par `tools/test-generate-sbom`. **Reste** : l'audit Dart n'a pas encore tourné (OSV-Scanner absent de cette machine ; son téléchargement et l'envoi de la liste des paquets à osv.dev sortent des règles réseau du projet, il tournera donc dans le workflow GitHub).

**Plateformes.** Toutes (c'est la chaîne de fabrication).

**Correctif.**
1. Ajouter `cargo audit` (ou `cargo deny check advisories`) dans une tâche planifiée hebdomadaire et avant chaque release, puisque l'exclusion de la CI courante est motivée par l'absence de réseau.
2. Ajouter un scan des dépendances Dart (par exemple OSV-Scanner sur `pubspec.lock`).
3. Activer Dependabot ou Renovate, et publier un SBOM avec chaque release.

---

## 4. Données de mail non chiffrées sur le disque — gravité MOYENNE (décidé : documenté)

**Le problème.** La base locale est du SQLite ordinaire (`rusqlite` avec `bundled`, aucun chiffrement). Les mails, les brouillons, les contacts et la mémoire de l'IA y sont lisibles par quiconque accède au fichier.

**Pourquoi c'est un risque.** Un ordinateur volé ou perdu sans chiffrement de disque, une sauvegarde copiée, un autre programme qui lit le dossier de l'utilisateur : tout le courrier est exposé. Les secrets (mots de passe, jetons) ne sont pas dans la base : ils sont dans le trousseau, ce qui est bien.

**Plateformes.** Linux, Windows, macOS. **Android** est protégé par l'isolation des applications (et `allowBackup="false"`).

**Décision (2026-11-26) : option B, ne pas chiffrer et documenter.** Raison : le gain de SQLCipher est partiel (un programme sous le même compte demande la clé au trousseau) pour un coût réel (build multi-plateforme, perte de la base si la clé du trousseau disparaît, migration des bases existantes). Fait : encart « Privacy and security » dans le `README`, rubrique dans `SECURITY.md`. Retour arrière : SQLCipher reste possible plus tard.

**Options examinées.**
- chiffrer la base avec SQLCipher, clé conservée dans le trousseau (coût : build et performances) ;
- ne pas chiffrer, mais **le documenter clairement** et recommander le chiffrement du disque (**retenu**). C'est ce que font la plupart des clients mail.

---

## 5. Windows : protection du jeton du démon à confirmer — gravité MOYENNE (reporté)

**Décision (2026-11-26) : reporté**, à faire quand une machine Windows à deux comptes sera disponible ; un correctif d'ACL non testé pourrait empêcher le démon de démarrer. Le risque est signalé aux utilisateurs dans `README.md` et `SECURITY.md` (« Known limitations »). **Reporté, à faire sous Windows.** Sous Linux le jeton et le socket sont réservés à l'utilisateur ; sous Windows la vérification est vide (`token/file.rs`, « phase Windows ») et le démon écoute en TCP sur `127.0.0.1`. Ce point demande une machine Windows à deux comptes pour être validé : problème, pistes de correctif et tests sont dans **`docs/securite/A-FAIRE-WINDOWS.md`**. Décision du 2026-11-22 : ne pas l'implémenter depuis Linux.

---

## 6. Le démon accepte un nombre illimité de connexions — gravité BASSE (corrigé)

**Statut (2026-11-22).** La première passe affirmait l'absence totale de plafond ; c'était inexact pour les connexions qui n'ont pas encore envoyé leur `hello` (plafonnées, avec délai de 5 s et taille maximale de trame), mais exact pour les connexions déjà identifiées. **Corrigé** (`server/context.rs`, `session/mod.rs`) : au plus **3 clients identifiés** en même temps (l'application n'en ouvre qu'un, multiplexé), les suivants sont fermés ; la place d'un client parti est réutilisable. Les connexions en attente de `hello` passent de 64 à **8**. Tests : `session_max_clients.rs` (2), `session_hello_limits.rs` (existant). **Compromis levé (2026-11-26, décision de l'utilisateur) :** à saturation des 8 places d'attente, la **plus ancienne** connexion en attente est évincée et la nouvelle est servie (`server/pending.rs`, `session/greeting.rs`) ; avant, la nouvelle était refusée et un programme local qui rouvrait 8 connexions muettes toutes les 5 s empêchait l'application de se connecter. Tests : `session_pending_eviction.rs` (3, stables sur 12 exécutions) ; l'ancien test « les suivantes sont refusées » est remplacé. **Reste** : un envoi de connexions plus rapide que la poignée de main d'un client légitime peut encore l'évincer (rafale d'au moins 8 connexions pendant quelques millisecondes), sans rien lire sans le jeton ; sous Windows (TCP sur `127.0.0.1`) n'importe quel compte local peut le tenter, ce que le tube nommé de `A-FAIRE-WINDOWS.md` supprimerait. De plus, le plafond est pris après l'envoi de `hello_ok` : un 4e client se croit connecté puis est fermé — **design du refus `busy` proposé (`DESIGN-BUSY.md`), en attente d'accord, non codé**. **Tests de l'éviction rendus déterministes (2026-11-26, SEC-04)** : sans leurs délais, ils échouaient 29 fois sur 30 ; le test attend maintenant que le démon ait inscrit chaque connexion (`PendingHellos::registered`, `support/arrival.rs`) ; 40 exécutions consécutives vertes.

---

## 7. Les identifiants OAuth embarqués ne sont que masqués — gravité BASSE (documenté)

**Statut (2026-11-26).** Aucun changement de mécanisme, c'est la recommandation. Documenté ici et dans `SECURITY.md` : ce secret est un simple masquage, il ne protège rien ; la protection vient de PKCE. Règle à garder : ne jamais mettre dans `oauth_embedded.rs` un secret réellement confidentiel.

**Le problème.** Le secret client OAuth est chiffré dans le binaire, mais la clé de déchiffrement est livrée juste à côté (`mira-assembly/src/oauth_embedded.rs`). N'importe qui peut donc le retrouver.

**Pourquoi c'est un risque limité.** Pour une application installée sur l'ordinateur de chacun, ce n'est pas un vrai secret : Google et Microsoft le savent. La protection réelle vient de PKCE, qui est en place. Le danger serait de s'imaginer que ce secret protège quelque chose.

**Plateformes.** Toutes.

**Correctif.** Ne rien changer au mécanisme, mais le documenter comme simple masquage, ne jamais y mettre de secret réellement confidentiel, et préférer un client « public » sans secret quand le fournisseur le permet.

---

## 8. Pas d'alerte d'hameçonnage sur les liens — gravité BASSE (fonction manquante)

**Le problème.** Quand le texte d'un lien affiche `banque.fr` et que la cible est `banque-secure.xyz`, rien ne prévient l'utilisateur. Idem pour les adresses à caractères trompeurs (homographes).

**Plateformes.** Toutes.

**Correctif.** Avant d'ouvrir un lien, comparer texte visible et domaine cible ; afficher la vraie adresse et demander confirmation en cas d'écart. Ajouter un avertissement sur les expéditeurs dont l'authentification échoue.

---

## 9. Manques de documentation et de processus — gravité INFORMATIVE

- ~~Pas de `SECURITY.md`~~ : écrit (2026-11-26), signalement privé par GitHub. À activer côté dépôt : *Settings → Code security → Private vulnerability reporting*.
- ~~Pas de modèle de menaces~~ : écrit, `MODELE-DE-MENACES.md`.
- Pas de **fuzzing** (MIME, IMAP, HTML, YAML, protocole du démon). **Reporté (2026-11-26)** : demande la toolchain nightly, de l'infrastructure et du tri ; aucun problème démontré (les tests de structures hostiles passent). À traiter comme un chantier à part, après la livraison des corrections actuelles.
- Pas de **journal d'audit** (échecs de connexion au démon, ajout de compte, envois). **Reporté (2026-11-26)** : n'empêche aucune attaque, et un journal mal conçu risque d'enregistrer des données sensibles ; demande une conception (quoi consigner, rotation, taille). À reprendre si MIRA gagne un usage multi-utilisateur ou un besoin de diagnostic.
- **Android** : le manifeste est correct (`allowBackup="false"`, pas de trafic non chiffré, secrets dans le Keystore, une seule activité exportée). Les réglages de la WebView reposent sur les valeurs par défaut ; preuve sur appareil à faire (`VERIFICATIONS-REELLES.md`).

**Correctif.** Les écrire ; le `SECURITY.md` et le modèle de menaces prennent une demi-journée.

---

## 10. Les releases ne sont pas signées et la CI n'est pas épinglée — gravité MOYENNE (CI épinglée ; signature reportée)

**Le problème.** `SHA256SUMS` est publié au même endroit que les paquets : il protège d'un téléchargement corrompu, pas d'une release remplacée (compte GitHub compromis, brèche de la CI). `install.sh` ne vérifie que cette somme. Les APK sont signés avec la clé de debug (`packaging/release.sh:173`). Les workflows référencent les actions par étiquette (`@v4`), et `securite.yml` télécharge OSV-Scanner en `releases/latest`, sans version ni somme : l'outil de surveillance est lui-même un maillon non vérifié.

**Plateformes.** Toutes (chaîne de livraison).

**Correctif proposé.**
1. Épingler les actions par SHA de commit et OSV-Scanner par version et somme (demande de consulter les versions : à faire avec accès au web).
2. Signer les releases (par exemple `minisign` ou signature Sigstore du `SHA256SUMS`) et faire vérifier la signature par `install.sh`.
3. Pour Android : une clé de signature propre, conservée hors du dépôt.

**Statut (2026-11-26).** Décision de l'utilisateur : suivre la recommandation.
- **Fait : épinglage.** Les 33 références d'actions des quatre workflows sont épinglées par SHA de commit (version en commentaire, SHA lus par `gh api`, pas recopiés) ; `cargo-deny` 0.20.2 (version éprouvée ici) et `cargo-ndk` 4.1.2 installés à version fixe ; OSV-Scanner v2.7.1 téléchargé à version fixe et vérifié par SHA-256. `tools/check-ci-pins` (et son auto-test) est branché sur `ci.sh` : une action sur étiquette, un `cargo install` libre ou un « latest » fait échouer la CI. Dependabot surveille aussi `/bridge` (absent jusqu'ici, ce qui a favorisé le constat 12) et met à jour les SHA épinglés. Non épinglés : les paquets `apt` et `choco` des runners.
- **Non fait, décision reportée : signature des releases** (minisign ou Sigstore sur `SHA256SUMS`, vérifiée par `install.sh`) et **clé de signature Android propre**. Raison : la signature demande une clé à conserver hors du dépôt (perdue, on ne signe plus ; volée, elle ne vaut plus rien) et change la procédure de release ; le gain dépend de la diffusion (faible tant que MIRA reste installé par l'auteur et quelques proches, important s'il devient public). **Déclencheur de reprise : la première diffusion publique, ou un tiers qui installe MIRA.**
- **À vérifier au prochain passage de la CI GitHub** : que les SHA épinglés fonctionnent (je n'ai pas pu lancer les workflows depuis cette machine).

---

## 11. Vue web durcie non branchée (à trancher), arrière-plans CSS distants chargés (corrigé) — gravité MOYENNE (découvert à la relecture, 2026-11-26)

**Ce que la relecture indépendante a établi (vérifié dans le code).**
- `WebMailBody` (la vue web du constat 1) **n'est appelée nulle part** : son appel dans `html_body.dart` a été retiré par le commit `33173df` (25 octobre, intitulé « compteurs non lus corrigés… »), ce qui ressemble à un retrait accidentel. Le rendu réel est `MailHtml`, en widgets natifs (`flutter_widget_from_html`), sur le HTML d'origine nettoyé par `sanitize_original_html`. Les correctifs a et b du constat 1 et leurs tests sont justes mais protègent un chemin dormant ; **le nettoyage moteur (c) est, en pratique, la protection active**. `reader.dart` calcule encore `nativeBody` à partir de `usesWebView`.
- **Images distantes** : `MailHtml` ne filtre que `<img>`. `flutter_widget_from_html_core` 0.17.4 charge aussi `background-image:url(…)` (`buildDecorationImage` → `imageProviderFromNetwork`), et `MailWidgetFactory` ne le surcharge pas ; `sanitize_original_html` garde `style` et `<style>`. Un mail avec `<td style="background-image:url(https://…/u?id=…)">` fait donc partir un pixel espion à l'ouverture, quelle que soit la politique d'images. Existait avant la branche.

**Arrière-plans distants : corrigé (2026-11-26).** `MailWidgetFactory(remoteImages: …)` surcharge `imageProviderFromNetwork` : refusé tant que les images distantes ne sont pas autorisées pour le message. Test de non-régression écrit d'abord et vu en échec (`html_remote_background_test.dart`), famille `html_*_test.dart` verte (33 tests). Non couverts, à vérifier : les autres usages d'URL par le moteur de rendu (`list-style-image`, polices).

**Décision restante.** Rebrancher la vue web (si le retrait n'était pas voulu) ou assumer le rendu natif et réécrire le constat 1 en conséquence.

---

## 12. Le pont livré avait son propre `Cargo.lock`, resté en retard — gravité HAUTE (corrigé, 2026-11-26)

**Le problème.** `bridge/` (la bibliothèque `libmira_bridge.so` que charge l'application) a son **propre** `Cargo.lock`, distinct de celui de `engine/`. Les mises à jour de sécurité du constat 3 n'avaient été faites que dans `engine/Cargo.lock` : le pont embarquait encore `ammonia` 4.2.3 (le nettoyeur HTML, XSS corrigés en 4.2.5), `lettre` 0.11.23 (injection SMTP, corrigée en .24), `rustls-webpki` 0.103.15, `tokio` 1.53.3, `url` 2.5.8, `zstd` 0.14.2, `webpki-roots` 1.0.9, `ctutils` 0.4.3 (RUSTSEC-2026-0385), et SQLite non corrigé. Découvert en construisant le paquet de test : la construction réécrivait ce verrou. **Ce qui est livré aux utilisateurs, c'est le pont : les corrections du constat 3 ne les avaient donc pas atteints.**

**Correctif.** `cargo update` ciblé dans `bridge/` sur les mêmes versions que le moteur ; `cargo deny check advisories` vert sur les deux verrous (hors avis « non maintenu » déjà justifiés dans `engine/deny.toml`). `securite.yml` contrôle désormais aussi le pont. `tools/generate-sbom` réunit les deux verrous (pont et démon).

**Reste.** Aucun test automatique n'empêche les deux verrous de diverger de nouveau ; seul le workflow hebdomadaire le détecterait. Piste : faire contrôler les avis du pont dans `ci.sh` quand le réseau est disponible, ou un test qui compare les versions des dépendances communes.

---

## 13. Les secrets en mémoire n'étaient pas effacés à leur destruction — gravité BASSE (corrigé en partie, 2026-11-26)

**Le problème.** Aucun usage de `zeroize` dans le dépôt : `SecretValue` (mots de passe, jetons, clés d'IA lus du trousseau) restait en mémoire après sa destruction jusqu'à réutilisation de la page. Exploitable seulement par un processus qui lit la mémoire du démon (même utilisateur ou vidage mémoire), donc faible.

**Correctif.** `SecretValue` implémente `Zeroize` et `ZeroizeOnDrop` (`mira-core/src/ports/secrets/value.rs`) ; `zeroize` 1.9.1 était déjà dans les deux verrous, seule la liste de dépendances de `mira-core` change (une ligne dans `engine/Cargo.lock` et `bridge/Cargo.lock`). Tests : `ports_secret_value_zeroize.rs` (3).

**Reste.** Les copies hors de `SecretValue` ne sont pas effacées : chaînes du mot de passe saisi dans une requête de l'API avant son rangement, tampons internes de la bibliothèque du trousseau et du TLS, `Token` du démon (`String`), jeton lu côté Dart. Ce n'est pas corrigeable à coût raisonnable ; risque accepté (même frontière que le constat 4).

---

## 14. Une image distante autorisée pouvait viser une machine locale — gravité BASSE (corrigé en partie, 2026-11-26)

**Le problème.** Une fois le contenu distant autorisé pour un message ou un expéditeur (jamais par défaut), rien ne filtrait l'hôte des images : `<img src="http://192.168.1.1/…">` ou `http://127.0.0.1:port/…` faisait émettre par l'ordinateur de l'utilisateur une requête GET aveugle vers son réseau local (équipement à interface web, service local), la règle CSP de la vue web (`img-src data: https: http:`) et le moteur de widgets laissant tout passer. Découvert en instruisant SEC-A10-3.

**Correctif.** Les deux assainisseurs du moteur retirent l'attribut qui chargerait automatiquement (`img src`, `srcset`, `video poster`, `background`…) une ressource dont l'hôte est une adresse locale, privée ou réservée, ou un nom sans domaine public (`localhost`, `.local`, `.lan`…), **même avec l'accord de l'utilisateur** : `html/sanitize.rs` (widgets ; la ressource est consignée dans `blocked_resources`) et `html/original.rs` (vue web). Mêmes plages que le filtre du désabonnement (`mime/unsubscribe_ranges.rs`). Tests : `html_remote_private_targets.rs` (4, vus rouges d'abord).

**Reste.** Un nom public qui se résout vers une adresse privée (rebinding DNS) n'est pas détecté : le moteur ne résout pas les noms. Les arrière-plans écrits dans un attribut `style` ne sont pas filtrés par hôte. `http:` en clair reste admis pour une image autorisée.

---

## Points déjà bons (pour mémoire)

- Jeton du démon comparé en temps constant (HMAC) ; **correction de ma première passe** qui l'avait signalé à vérifier.
- Le moteur refuse toute connexion en clair, sans contournement de certificat.
- Android : secrets chiffrés par une clé du Keystore.
- L'IA ne fait que **proposer** (envoi, suppression) ; les mails, IBAN, cartes et codes à usage unique sont masqués avant envoi (`ai/masking`).
- Archives extraites avec contrôle des chemins et des liens, limite de taille.
- Téléchargement des modèles IA vérifié par somme de contrôle.
- URL des connecteurs IA nettoyées : pas d'identifiants dans l'adresse, pas de clair hors machine locale.

## Tests demandés par la première passe — résultats (2026-11-26)

| Sujet | Résultat | Preuve |
|---|---|---|
| Injection d'en-têtes (CRLF) dans sujet, nom, adresse | **Sûr** : valeurs mises sur une ligne, adresses piégées refusées | `smtp_compose_validation.rs` (existant) |
| Idem pour nom et type des pièces jointes | **Sûr** | `smtp_compose_attachment_injection.rs` (nouveau) |
| Injection de commandes IMAP | **Sûr par lecture du code** : `quote`/`quote_mailbox` refusent CR, LF, NUL et échappent `"` et `\\` ; drapeaux en énumération fixe ; recherche UTF-8 par littéral | `imap/protocol/quoting.rs`, `imap/operations/*` |
| YAML des règles géant ou imbriqué | **Sûr** : 2 Mo, 500 éléments, alias et doublons refusés ; 100 000 niveaux refusés sur une pile de 512 Ko | `portability_envelope_limits.rs`, `portability_envelope_depth.rs` (nouveau) |
| MIME à 10 000 parties ou 5 000 niveaux | **Sûr** : lu en moins de 10 s (borne du test), pas de dépassement de pile | `mime_structure_limits.rs` (nouveau) |
| Injection de prompt par un mail | **Atténué par conception** : aucun outil d'envoi ou de suppression, uniquement des propositions validées par clic (`remember`, brouillon, action) ; consentement par type de contenu pour les connexions distantes ; réponses rendues sans image ni lien (`chat_markdown.dart`) | `ai/tools/catalog.rs`, `ai_tools_loop_consent.rs`. Reste un test de bout en bout avec un mail hostile |
| URL de base IA éditable | **Acceptable** : refus de `file:`, du clair hors boucle locale, des identifiants ; aucune redirection. Une adresse HTTPS du réseau local reste permise (serveur auto-hébergé) | `ai/base_url.rs`, `ai/transport/client.rs` |
| Réglages de la WebView Android | **Lu, sain par défaut** : manifeste (une activité exportée, sauvegardes exclues, pas de clair), chargement par `loadHtmlString`. Réglages non explicites ; preuve sur appareil à faire | `app/android/.../AndroidManifest.xml`, `web_mail_body.dart`, `VERIFICATIONS-REELLES.md` §4 |

## Ordre de correction conseillé

1. **Constat 1** (vue web) : c'est le seul qui touche directement un attaquant anonyme par un simple mail.
2. **Constat 3** (surveillance des dépendances) : rapide, grand gain.
3. **Constat 2** (désabonnement).
4. **Constat 5** (Windows) : reporté, voir `A-FAIRE-WINDOWS.md`.
5. Constats 7 à 9, et la décision sur le chiffrement de la base (4).
