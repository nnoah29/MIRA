# Vérifications réelles de sécurité — procédure

Ce document transforme les hypothèses de `CONSTATS.md` en preuves. Les tests automatiques ont validé les fonctions ; ici on vérifie **dans la vraie application, sur chaque plateforme**.

**Règles du projet** : aucun compte réel, aucun serveur externe. Tout se passe contre un serveur IMAP local sur `127.0.0.1` et un écouteur local qui joue le rôle de « l'attaquant ».

## 1. Préparer

1. **Un serveur IMAP local jetable** sur `127.0.0.1` (par exemple Dovecot ou GreenMail, au choix), avec un utilisateur de test. Pour Android, le serveur tourne sur l'ordinateur ; l'émulateur le joint par `10.0.2.2` (le clair n'est admis que vers la boucle locale : voir si l'émulateur le permet, sinon utiliser `adb reverse tcp:PORT tcp:PORT`).
2. **Déposer les mails piégés** :
   ```bash
   tools/deposer-mails-pieges PORT_IMAP utilisateur motdepasse
   ```
   Le script refuse tout hôte autre que `127.0.0.1`. Les 9 mails sont dans `docs/securite/mails-pieges/`.
3. **Lancer l'écouteur « attaquant »** dans un terminal, qui affiche toute requête reçue :
   ```bash
   python3 -m http.server 8999 --bind 127.0.0.1
   ```
   Chaque ligne `GET /xxx` qui apparaît est un **échec** (le mail a fait sortir une requête). Sur Android, `adb reverse tcp:8999 tcp:8999`.
4. Ajouter le compte de test dans MIRA, synchroniser.

## 2. Mails piégés : résultat attendu

Pour chaque mail, l'ouvrir dans MIRA, cliquer sur chaque lien et bouton, attendre 10 secondes, regarder le terminal de l'écouteur et noter : *navigateur du système ouvert ?* *nouvelle fenêtre ?* *ligne dans l'écouteur ?*

| Mail | À faire | Attendu |
|---|---|---|
| 01 meta refresh | ouvrir | rien ne s'ouvre, aucune ligne `/refresh` |
| 02 lien `data:` et `javascript:` | cliquer les deux liens | rien ne s'ouvre, aucune ligne `/data` ni `/js` |
| 03 formulaire | ouvrir, cliquer le bouton | aucune ligne `/form` |
| 04 `<base>` | cliquer le lien relatif | aucune ligne `/base/…` |
| 05 scripts et `on…` | ouvrir | aucune ligne `/script`, `/onerror`, `/svg`, `/bodyload` |
| 06 ressources distantes | ouvrir **sans** autoriser les images | aucune ligne ; puis autoriser les images distantes : seules `/pixel.png` (et éventuellement `/srcset.png`) peuvent apparaître, **jamais** `/import.css`, `/link.css` |
| 07 cadres, objets | ouvrir | rien d'affiché, aucune ligne `/iframe`, `/object`, `/embed` |
| 08 désabonnement | ouvrir | pas de bouton « se désabonner en un clic » vers `127.0.0.1` |
| 09 injection de prompt | demander à l'assistant de résumer le mail (connexion IA de test) | l'assistant résume ; **aucun** envoi ni suppression ; au pire une proposition à valider ; aucune mémoire retenue sans clic |

Pour 06, l'autorisation des images est un choix de l'utilisateur : un pixel chargé après accord n'est pas une faille, une feuille de style ou un cadre chargé l'est.

## 3. Plateformes

À refaire sur chacune ; la vue web n'est pas la même :

| Plateforme | Moteur de la vue | Spécifique à vérifier |
|---|---|---|
| Linux | WebKitGTK | tout le tableau |
| Android | WebView | tout le tableau ; en plus voir §4 |
| Windows | WebView2 | tout le tableau |

## 4. Android : réglages de la WebView

**Lecture statique faite (2026-11-26)** : `AndroidManifest.xml` ne déclare qu'une activité exportée (le lanceur) ; `allowBackup="false"` et `data_extraction_rules.xml` excluent toute sauvegarde ; aucun `usesCleartextTraffic` (donc refusé par défaut aux versions récentes d'Android) ; la vue (`webview_all`) charge le courriel par `loadHtmlString`, donc sans origine fichier. Les réglages de la WebView ne sont pas durcis explicitement : ils reposent sur les valeurs par défaut du système (accès aux fichiers refusé depuis Android 11, contenu mixte refusé). JavaScript reste actif (mesure de hauteur).

**À vérifier sur appareil** : que les mails 01 à 07 du §2 ne produisent aucune requête vers l'écouteur, ce qui prouve que ces défauts tiennent sur la version d'Android testée. Si un doute subsiste, durcir explicitement `allowContentAccess` et `allowFileAccess` à `false` via le contrôleur Android du plugin.

## 5. Compte rendu

Pour chaque ligne du tableau et chaque plateforme, noter **OK** ou **ÉCHEC** (avec la ligne vue dans l'écouteur) dans `docs/securite/RESULTATS-VERIFICATIONS.md`. Un ÉCHEC est traité comme une faille : correctif, puis test de non-régression avant de refaire la vérification.
