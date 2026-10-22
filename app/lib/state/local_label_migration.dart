import 'package:shared_preferences/shared_preferences.dart';

import '../engine/local_label_admin.dart';
import 'local_labels.dart';

/// Reprend dans le moteur les libellés locaux gardés dans les préférences, une seule fois.
/// Rend `true` si tout est repris (ou rien à reprendre) ; `false` en cas d'échec, copie locale intacte.
Future<bool> migrateLocalLabels(
  SharedPreferences prefs,
  LocalLabelAdmin admin,
) async {
  final raw = prefs.getString('localLabels');
  if (raw == null || prefs.getBool('localLabelsMigrated') == true) return true;
  final old = LocalLabels.decode(raw);
  try {
    if (old.defs.isNotEmpty) {
      await admin.importLabels([
        for (final e in old.defs.entries)
          LocalLabelImport(e.value.n, e.value.c, old.conversationsOf(e.key)),
      ]);
    }
  } catch (_) {
    return false;
  }
  await prefs.setBool('localLabelsMigrated', true);
  await prefs.remove('localLabels');
  return true;
}
