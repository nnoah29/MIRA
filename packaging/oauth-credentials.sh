# Sourcé par les scripts de build : désigne le fichier dont la section [oauth] est chiffrée dans le moteur (build.rs de mira-assembly).
# Par défaut ~/.config/mira/config.toml ; MIRA_OAUTH_CONFIG= (vide) construit des paquets sans identifiants.
if [ -z "${MIRA_OAUTH_CONFIG+x}" ] && [ -f "${XDG_CONFIG_HOME:-$HOME/.config}/mira/config.toml" ]; then
  export MIRA_OAUTH_CONFIG="${XDG_CONFIG_HOME:-$HOME/.config}/mira/config.toml"
fi
if [ -n "${MIRA_OAUTH_CONFIG:-}" ]; then
  echo "Identifiants OAuth intégrés (chiffrés) depuis $MIRA_OAUTH_CONFIG"
else
  unset MIRA_OAUTH_CONFIG
  echo "Aucun identifiant OAuth intégré : la connexion OAuth demandera un fichier config.toml"
fi
