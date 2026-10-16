# MIRA — interface Flutter (données fictives pour l’instant)

Portage fidèle du client mail HTML/CSS/JS (le prototype HTML/CSS/JS d’origine) : même design, mêmes
fonctionnalités, mêmes **données fictives** (aucun backend, rien n'est envoyé nulle part).

## Lancer

```bash
flutter pub get
flutter run -d linux        # ou : -d chrome, -d <appareil Android>
flutter test                # 40 tests (logique + parcours d'interface)
```

Les polices (Anton, Hanken Grotesk, JetBrains Mono) sont **embarquées** dans
`assets/google_fonts/` : l'application fonctionne hors ligne.

Aide au débogage : `flutter run -d linux --dart-define=KM_W=390 --dart-define=KM_H=844`
force une taille d'écran logique (pour tester les vues tablette / mobile sur le bureau).

## Ce qui est reproduit

| Domaine | Détail |
| --- | --- |
| Mise en page | Barre latérale 248 px, barre du haut, liste (430 px) + panneau droit ; ≤ 1100 px : une seule colonne ; ≤ 760 px : tiroir de navigation avec voile |
| Thèmes | Sombre (défaut) et clair, mêmes variables de couleur que le CSS |
| Dossiers | Réception, Favoris, Reportés, Envoyés, Brouillons, Archives, Spam, Corbeille + 4 libellés |
| Liste | Groupes par date collants, pastilles de filtre, tri, recherche (`/`), sélection multiple, actions groupées, annulation |
| Lecteur | Fils repliables, **lecture des pièces jointes** (visionneuse : PDF fictifs paginés avec zoom, images, fichiers texte réels, navigation ←/→, téléchargement simulé), répondre / à tous / transférer |
| Rédaction | Puces de destinataires + autocomplétion, Cc/Cci, expéditeur, éditeur riche (gras, italique, souligné, alignement, lien, liste), pièces jointes réelles, brouillon automatique, envoi programmé, annulation d'envoi, réponse automatique simulée |
| Assistant IA | Réponses simulées sur les vraies données (résumé, rédaction, urgent, non lus, traduction, nettoyage, recherche), frappe progressive |
| Comptes | Liste déroulante à côté d'« Assistant IA » : changer de compte (chaque compte a sa boîte fictive), **Toutes les boîtes** (boîte unifiée, avec l'étiquette du compte sur chaque message) ou ajouter un compte (nom + adresse, mémorisé) |
| Fond d'écran (Linux) | MIRA suit en direct `/tmp/qs_colors.json` (ou `$QS_COLORS_FILE`) généré pour ShellArie : fond, surfaces, texte et accent (teinte la plus vive de `~/.cache/theme-color/colors.css`). Réglage « Suivre / Ignorer » dans Paramètres › Apparence |
| Langue | Français / English (Paramètres › Apparence), mémorisée. Les textes sont écrits en français et servent de clés (`t('…')`) ; les traductions vivent dans `tool/en.py` → `lib/i18n/en.dart`. Pour ajouter une langue : un nouveau dictionnaire + une entrée dans `AppLang`. Les messages fictifs restent en français |
| Paramètres | Thème, densité, aperçus, nouveaux messages simulés, signature, réinitialisation (persistés) |
| Raccourcis | `j k Entrée c r a f e # s u x / ?` `Ctrl+Entrée` `Échap` |
| Divers | Popovers, toasts avec « Annuler », modale d'aide, mails entrants simulés, rappels des mails reportés |

## Architecture

```
lib/
  data/    modèles, mini-parseur HTML (le jeu de données garde le HTML d'origine), formats, données fictives
  state/   MailStore (tout l'état + les actions, port de app.js), assistant simulé, éditeur riche
  theme/   palettes sombre/claire (ThemeExtension), polices, icônes SVG
  ui/      coque responsive, barre latérale, liste, lecteur, composeur, assistant, paramètres, popovers/toasts
test/      store_test.dart (logique) et app_test.dart (parcours de widgets)
```

## Fidélité visuelle

Les écrans ont été comparés à l'original par captures (même viewport, mêmes états : liste,
lecteur, rédaction, réponse, assistant, paramètres, thème clair, popovers, mobile, tiroir).
Deux écarts de moteur sont compensés : Flutter arrondit chaque ligne de texte à l'entier
(`Fonts.snap`) et répartit l'interligne différemment du CSS (`leadingDistribution.even`).

## Différences connues

- Dans la rédaction, le texte cité d'une réponse est affiché (bordure orange comprise) mais
  n'est pas modifiable ; la zone de saisie couvre le texte avant la citation.
- Les sauts de paragraphe de l'éditeur utilisent une ligne de 16 px (marge d'un `<p>`) ;
  l'éditeur est un champ de texte avec styles par caractère, pas un `contenteditable`.
- Comme dans l'original, le bouton « retour » est masqué à toutes les largeurs ; sur Android,
  le geste/bouton retour du système ferme le panneau ouvert.
- L'ajout de pièces jointes utilise le sélecteur de fichiers natif (sous Linux : `zenity`/portail).
- La couleur du texte sélectionné ne change pas (seul le fond devient orange).
- La version Android n'a pas été compilée dans cet environnement (licences SDK non acceptées).
