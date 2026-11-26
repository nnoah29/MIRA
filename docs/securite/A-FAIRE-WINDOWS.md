# À faire sous Windows — sécurité du jeton du démon

Issu du constat 5 de l'audit (`CONSTATS.md`). **Non implémenté** : il demande une machine Windows réelle (au moins deux comptes) pour être validé. Le développeur principal travaille sous Linux ; ce document est un point de départ pour qui le fera.

**Décision du 2026-11-26 : reporté.** Déclencheur pour reprendre : disposer d'une machine Windows avec deux comptes, ou une première diffusion Windows sur postes partagés. Une fois corrigé, retirer la mention « Windows » de `README.md` (section « Privacy and security ») et la rubrique « Known limitations » de `SECURITY.md`.

## Le problème

Sous Linux, le fichier du jeton est en 0600, son dossier en 0700 et le socket est réservé à l'utilisateur. Sous Windows, aucun de ces contrôles n'existe :

- `engine/crates/mira-daemon/src/token/file.rs`, `refuse_shared` (variante `#[cfg(not(unix))]`) renvoie toujours `Ok(())` : un fichier de jeton lisible par d'autres comptes n'est pas refusé.
- `token/publish.rs` : `create_parent` et `write_private` ne fixent des droits que sous `#[cfg(unix)]`. Sous Windows, le fichier hérite des droits du dossier parent.
- `transport/port_file.rs` : même chose pour le dossier du fichier de port.
- Le démon écoute en TCP sur `127.0.0.1` : **tout processus local, de n'importe quel compte** (autre session, bureau à distance, poste partagé) peut s'y connecter. Seul le jeton (256 bits, comparé en temps constant) l'arrête.

## Pourquoi c'est important

Le jeton est la seule clé d'accès au démon, donc à tous les mails et aux envois. Si le dossier qui le contient est lisible par un autre compte (hérité d'un dossier trop ouvert), l'autre compte lit le jeton et prend le contrôle.

À noter : le dossier de profil habituel (`%LOCALAPPDATA%`) est déjà privé à l'utilisateur par défaut. Le risque réel est donc surtout un profil placé dans un dossier partagé, ou un compte administrateur. Gravité moyenne, **à confirmer par un test**.

## Pistes de correctif (à trancher)

1. **ACL explicites** : à la création du dossier et du fichier, supprimer l'héritage et n'accorder l'accès qu'au SID de l'utilisateur courant (API `SetNamedSecurityInfoW` / `SECURITY_ATTRIBUTES`, ou la crate `windows` ; l'équivalent de `refuse_shared` lit le DACL et refuse tout ACE accordant la lecture à un autre SID que l'utilisateur, `SYSTEM` ou les administrateurs). Le projet interdit `unsafe` : prévoir une crate qui l'encapsule, ou isoler l'appel et en discuter dans `docs/dev/DECISIONS-AGENT.md`.
2. **Tube nommé (named pipe)** à la place du TCP : `\\.\pipe\mira-<profil>` avec un descripteur de sécurité limité à l'utilisateur. Supprime l'exposition réseau locale ; plus de travail côté client Dart et démon (nouveau `Listener` dans `transport/`).
3. **Contrôle du pair TCP** : plus fragile, à éviter.

Recommandation : (1) d'abord, car c'est petit et couvre le jeton ; (2) ensuite, si l'on veut fermer aussi le port.

## Tests à faire (sur Windows)

- Créer un second compte utilisateur local ; vérifier qu'il ne peut ni lire le fichier du jeton ni le dossier du profil (`icacls <chemin>` doit montrer l'utilisateur seul, `SYSTEM` et `Administrators`).
- Avec ce second compte, se connecter au port TCP du démon sans jeton, puis avec un faux jeton : refus et fermeture (comportement déjà testé sous Linux, `session_hello*`).
- Écrire un test d'intégration `#[cfg(windows)]` pour la variante `refuse_shared` : un fichier dont le DACL est ouvert à `Everyone` est refusé.
- Vérifier que le démon démarre encore avec un profil par défaut (aucune régression d'installation).

## Pour macOS

À traiter dès le départ : socket Unix dans le dossier de l'utilisateur (0600/0700), pas de TCP.

## À ajouter (relecture du 2026-11-26)

Sous TCP sur `127.0.0.1`, n'importe quel compte local peut ouvrir des connexions muettes sans jeton. Depuis le 2026-11-26 la plus ancienne connexion en attente est évincée à saturation (`server/pending.rs`) : le blocage durable de l'application est levé, mais une rafale d'au moins 8 connexions pendant la poignée de main d'un client légitime peut encore l'évincer, et rien n'arrête le bruit. Un tube nommé réservé à l'utilisateur supprime ce point. À tester avec le second compte.
