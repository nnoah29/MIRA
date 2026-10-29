//! Journal des envois à un service d'IA : lignes chargées par pages de 50, plus récentes d'abord.

import 'package:flutter/foundation.dart';

import '../engine/ai_admin.dart';
import '../i18n/engine_errors.dart';

/// Nombre de lignes demandées au moteur à chaque page.
const aiJournalPage = 50;

/// Lignes du journal, chargement par pages, effacement ; une erreur du moteur devient un texte traduit.
class AiJournalState extends ChangeNotifier {
  final AiAdmin admin;
  List<AiJournalRow> rows = const [];
  bool loaded = false;
  bool loading = false;
  bool hasMore = false;
  String? error;
  bool _disposed = false;

  AiJournalState(this.admin);

  /// Charge la première page (les plus récentes).
  Future<void> load() => _fetch(replace: true);

  /// Ajoute la page suivante, après la dernière ligne connue.
  Future<void> more() => _fetch(replace: false);

  /// Efface tout le journal puis le relit.
  Future<void> clear() async {
    try {
      await admin.clearJournal();
      error = null;
    } on Object catch (e) {
      error = engineErrorText(e);
    }
    await load();
  }

  Future<void> _fetch({required bool replace}) async {
    if (loading) return;
    loading = true;
    _changed();
    try {
      final before = replace || rows.isEmpty ? null : rows.last.id;
      final page = await admin.journal(limit: aiJournalPage, before: before);
      rows = replace ? page : [...rows, ...page];
      hasMore = page.length == aiJournalPage;
      error = null;
    } on Object catch (e) {
      error = engineErrorText(e);
    }
    loading = false;
    loaded = true;
    _changed();
  }

  void _changed() {
    if (!_disposed) notifyListeners();
  }

  @override
  void dispose() {
    _disposed = true;
    super.dispose();
  }
}
