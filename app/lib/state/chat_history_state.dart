//! Liste des conversations enregistrées avec l'assistant IA : pages de 50, renommage, suppression, interrupteur.

import 'package:flutter/foundation.dart';

import '../engine/ai_history_admin.dart';
import '../i18n/engine_errors.dart';

/// Nombre de conversations demandées au moteur à chaque page.
const chatHistoryPage = 50;

/// Conversations enregistrées, chargées par pages ; une erreur du moteur devient un texte traduit.
class ChatHistoryState extends ChangeNotifier {
  final AiHistoryAdmin? Function() admin;
  List<AiConversationMeta> items = const [];
  bool loaded = false;
  bool loading = false;
  bool hasMore = false;
  bool enabled = true;
  String? error;
  int _generation = 0;
  bool _disposed = false;

  ChatHistoryState(this.admin);

  /// Faux sans moteur (démonstration) : il n'y a alors pas d'historique.
  bool get available => admin() != null;

  /// Charge l'interrupteur et la première page (les plus récentes) ; une lecture déjà en vol devient caduque.
  Future<void> load() => _fetch(replace: true);

  /// Ajoute la page suivante, après la dernière conversation connue.
  Future<void> more() => _fetch(replace: false);

  /// Renomme une conversation puis relit la liste.
  Future<void> rename(String id, String title) =>
      _change((ai) => ai.rename(id, title));

  /// Supprime une conversation puis relit la liste.
  Future<void> delete(String id) => _change((ai) => ai.delete(id));

  /// Supprime toutes les conversations puis relit la liste.
  Future<void> clearAll() => _change((ai) => ai.clearAll());

  /// Active ou coupe l'historique ; en cas d'échec la valeur précédente reste.
  Future<void> setEnabled(bool value) async {
    final ai = admin();
    if (ai == null) return;
    try {
      enabled = await ai.setEnabled(value);
      error = null;
    } on Object catch (e) {
      error = engineErrorText(e);
    }
    _changed();
  }

  Future<void> _change(Future<Object?> Function(AiHistoryAdmin) action) async {
    final ai = admin();
    if (ai == null) return;
    _generation++;
    loading = false;
    String? failure;
    try {
      await action(ai);
    } on Object catch (e) {
      failure = engineErrorText(e);
    }
    await load();
    if (failure == null) return;
    error = failure;
    _changed();
  }

  Future<void> _fetch({required bool replace}) async {
    final ai = admin();
    if (ai == null) return _unavailable();
    if (!replace && (loading || items.isEmpty)) return;
    final generation = replace ? ++_generation : _generation;
    loading = true;
    _changed();
    final before = replace ? null : items.last.id;
    bool? on;
    List<AiConversationMeta>? page;
    String? failure;
    try {
      if (replace) on = await ai.enabled();
      page = await ai.list(limit: chatHistoryPage + 1, before: before);
    } on Object catch (e) {
      failure = engineErrorText(e);
    }
    if (generation != _generation) return;
    _apply(on, page, failure, replace);
  }

  void _unavailable() {
    items = const [];
    hasMore = false;
    error = null;
    loaded = true;
    _changed();
  }

  void _apply(
    bool? on,
    List<AiConversationMeta>? page,
    String? failure,
    bool replace,
  ) {
    if (on != null) enabled = on;
    if (page != null) {
      final kept = page.take(chatHistoryPage).toList();
      items = replace ? kept : [...items, ...kept];
      hasMore = page.length > chatHistoryPage;
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
