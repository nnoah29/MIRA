## 1. Hiérarchie des branches

- **main (La Forteresse)**

    Contient uniquement du code **stable et validé**.

    On **ne développe jamais directement dessus**.

- **dev (Le Laboratoire)**

    Branche d’intégration. Toutes les fonctionnalités y sont fusionnées.

    Elle doit **toujours compiler** et passer les checks.

- **feat/E-XX-nom**

    Une branche par tâche.

Exemples :

```bash
feat/E-05-csv-parser
feat/E-10-parallel-rayon
```

Ces branches sont **créées à partir de `dev`**.

- **fix/nom-du-bug**

    Pour corriger rapidement un bug détecté sur `dev`.

---

# 2. Rituel avant un push (obligatoire)

Avant d’envoyer votre code sur le dépôt distant, **vous devez exécuter cette séquence**.

### 1. Mettre la branche à jour avec `dev`

```bash
git pull --rebase origin dev
```

Cela permet d’intégrer le travail des autres **sans créer de commits de merge inutiles**.

---

### 2. Vérifier la qualité du code

```bash
make check
```

Cette commande exécute automatiquement :

- vérification du formatage du code (`cargo fmt --check`)
- lint Rust (`cargo clippy`)
- tests unitaires (`cargo test`)

Si une étape échoue, le code doit être corrigé avant d’être poussé.

---

### 3. Corriger le formatage si nécessaire

Si `make check` échoue à cause du formatage :

```bash
cargo fmt
```

Puis relancer :

```bash
make check
```

---

# 3. Gestion du travail en cours (WIP)

Si vous devez arrêter de coder en plein milieu d'une tâche mais que vous voulez sauvegarder votre travail :

Utilisez le préfixe **WIP** dans votre commit.

Exemple :

```bash
git commit -m "WIP: start csv streaming parser"
```

⚠️ Une branche contenant des commits **WIP ne doit jamais être fusionnée dans `dev`.**

---

# 4. Pull Requests & revue de code

Comme c’est un projet secondaire, le processus reste simple.

---

## Auto-validation

Si :

- la PR fait **moins de 50 lignes**
- `make check` passe
- personne ne s’y oppose

alors l’auteur peut fusionner la PR lui-même **après 4 heures**.

---

## Revue à deux

Pour les modules importants (exemples) :

- moteur de calcul
- pipeline de données
- système réseau
- architecture du moteur

une validation par **un autre membre de l’équipe** est requise.

---

## Commentaires de revue

On ne discute **pas** :

- du style de code
- du formatage

Ces points sont gérés automatiquement par :

- `cargo fmt`
- `cargo clippy`

Les discussions doivent porter sur :

- la logique
- l’architecture
- les performances
- la sécurité

---

# 5. Suppression des branches après merge

Une fois qu’une branche **feat/** ou **fix/** est fusionnée dans `dev`, il est recommandé de **la supprimer automatiquement** afin de garder le dépôt propre.

Si vous avez fusionné la PR :

```bash
git branch -d feat/E-10-parallel-rayon
git push origin --delete feat/E-10-parallel-rayon
```

La plupart des plateformes Git (GitHub, GitLab) permettent aussi **d’activer la suppression automatique des branches après merge**.

Objectif :

- éviter l’accumulation de branches inutiles
- garder un dépôt lisible
- faciliter la navigation dans le projet

---

# Résumé des commandes

## Commencer une tâche

```bash
git checkout dev
git pull origin dev
git checkout -b feat/E-10-parallel-rayon
```

---

## Développement

Coder normalement puis vérifier :

```bash
make check
```

---

## Commit

```bash
git add .
git commit -m "feat(engine): implement parallel dataset processing with rayon"
```

---

## Mettre à jour la branche

```bash
git pull --rebase origin dev
```

---

## Push

```bash
git push origin feat/E-10-parallel-rayon
```

---

# Workflow complet typique

```bash
git checkout dev
git pull origin dev
git checkout -b feat/E-10-parallel-rayon

# développement
make check

git add .
git commit -m "feat(engine): implement parallel dataset processing with rayon"

git pull --rebase origin dev
git push origin feat/E-10-parallel-rayon
```

---

# Objectif de ce workflow

Garantir que :

- le code est **formaté**
- il n’y a **aucun warning**
- les **tests passent**
- la branche **dev reste stable**
- l’historique git reste **propre et lisible**

---
