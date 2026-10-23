import 'dart:convert';

/// Dossiers de l'interface auxquels une boîte du serveur peut être rattachée.
const mappableFolders = ['inbox', 'sent', 'drafts', 'archive', 'spam', 'trash'];

/// Rattachements enregistrés (identifiant de boîte → dossier) ; vide si illisible.
Map<String, String> decodeFolderMap(String? raw) {
  if (raw == null) return {};
  try {
    final j = jsonDecode(raw) as Map<String, dynamic>;
    return {
      for (final e in j.entries)
        if (mappableFolders.contains(e.value)) e.key: '${e.value}',
    };
  } catch (_) {
    return {};
  }
}

/// Texte enregistré dans les préférences pour ces rattachements.
String encodeFolderMap(Map<String, String> map) => jsonEncode(map);

/// Rattachements enregistrés (identifiant de boîte → clé de libellé local) ; vide si illisible.
Map<String, String> decodeLabelMap(String? raw) {
  if (raw == null) return {};
  try {
    final j = jsonDecode(raw) as Map<String, dynamic>;
    return {for (final e in j.entries) e.key: '${e.value}'};
  } catch (_) {
    return {};
  }
}

/// Texte enregistré dans les préférences pour ces rattachements.
String encodeLabelMap(Map<String, String> map) => jsonEncode(map);
