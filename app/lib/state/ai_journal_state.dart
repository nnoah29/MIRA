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
  int _generation = 0;
  bool _disposed = false;

  AiJournalState(this.admin);

  /// Charge la première page (les plus récentes) ; une lecture déjà en vol devient caduque.
  Future<void> load() => _fetch(replace: true);

  /// Ajoute la page suivante, après la dernière ligne connue.
  Future<void> more() => _fetch(replace: false);

  /// Efface tout le journal puis le relit ; l'erreur d'un effacement raté reste affichée.
  Future<void> clear() async {
    _generation++;
    loading = false;
    String? failure;
    try {
      await admin.clearJournal();
    } on Object catch (e) {
      failure = engineErrorText(e);
    }
    await load();
    if (failure == null) return;
    error = failure;
    _changed();
  }

  Future<void> _fetch({required bool replace}) async {
    if (!replace && (loading || rows.isEmpty)) return;
    final generation = replace ? ++_generation : _generation;
    loading = true;
    _changed();
    final before = replace ? null : rows.last.id;
    List<AiJournalRow>? page;
    String? failure;
    try {
      page = await admin.journal(limit: aiJournalPage + 1, before: before);
    } on Object catch (e) {
      failure = engineErrorText(e);
    }
    if (generation != _generation) return;
    _apply(page, failure, replace);
  }

  void _apply(List<AiJournalRow>? page, String? failure, bool replace) {
    if (page != null) {
      final kept = page.take(aiJournalPage).toList();
      rows = replace ? kept : [...rows, ...kept];
      hasMore = page.length > aiJournalPage;
    }
    error = failure;
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
