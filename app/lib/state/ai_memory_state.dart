//! État de l'écran de la mémoire : réglages, installation du modèle, liste, recherche, modification et oubli des souvenirs.

import 'dart:async';

import 'package:flutter/foundation.dart';

import '../engine/ai_admin.dart';
import '../engine/ai_local_admin.dart';
import '../engine/ai_memory_admin.dart';
import '../i18n/engine_errors.dart';

/// Taille d'une page de souvenirs demandée au moteur.
const memoryPage = 50;

/// Ce que montre la section « Mémoire » des réglages ; rien n'est téléchargé ni calculé tant que l'utilisateur n'agit pas.
class AiMemoryState extends ChangeNotifier {
  final AiAdmin admin;
  final Duration pollEvery;
  AiMemoryState(
    this.admin, {
    this.pollEvery = const Duration(milliseconds: 500),
  });

  MemoryStatus? status;
  List<MemoryItem> items = const [];
  List<MemoryHit>? hits;
  bool loaded = false;
  bool hasMore = false;
  bool installing = false;
  double progress = 0;
  String? error;
  int? reindexed;
  String? _operation;
  bool _disposed = false;

  /// Dit si la mémoire est activée.
  bool get enabled => status?.settings.enabled ?? false;

  /// Relit le statut et la première page de souvenirs ; une erreur devient un texte traduit.
  Future<void> load() async {
    await _run(() async {
      status = await admin.memoryStatus();
      items = await admin.listMemories(limit: memoryPage);
      hasMore = items.length == memoryPage;
      loaded = true;
    });
  }

  /// Ajoute la page suivante, après le dernier souvenir connu.
  Future<void> more() => _run(() async {
    final page = await admin.listMemories(
      limit: memoryPage,
      after: items.isEmpty ? null : items.last.id,
    );
    items = [...items, ...page];
    hasMore = page.length == memoryPage;
  });

  /// Active ou coupe la mémoire ; couper ne supprime aucun souvenir.
  Future<void> setEnabled(bool value) => _settings(
    MemorySettings(
      enabled: value,
      connection: status?.settings.connection,
      model: status?.settings.model,
    ),
  );

  /// Calcule les embeddings sur l'appareil.
  Future<void> setLocal() => _settings(MemorySettings(enabled: enabled));

  /// Calcule les embeddings chez le service de [connection], avec ce [model] d'embeddings.
  Future<void> setProvider(String connection, String model) => _settings(
    MemorySettings(enabled: enabled, connection: connection, model: model),
  );

  Future<void> _settings(MemorySettings next) => _run(() async {
    status = await admin.setMemorySettings(next);
    await _refreshItems();
  });

  /// Lance l'installation du modèle d'embeddings puis suit son opération jusqu'à la fin.
  Future<void> install() async {
    error = null;
    progress = 0;
    installing = true;
    _changed();
    try {
      _operation = await admin.installMemoryModel();
    } on Object catch (e) {
      error = engineErrorText(e);
      installing = false;
      _changed();
      return;
    }
    await _poll();
  }

  Future<void> _poll() async {
    final operation = _operation;
    while (operation != null && !_disposed) {
      final OperationProgress info;
      try {
        info = await admin.operationProgress(operation);
      } on Object catch (e) {
        error = engineErrorText(e);
        break;
      }
      progress = info.fraction;
      if (!info.running) break;
      _changed();
      await Future<void>.delayed(pollEvery);
    }
    _operation = null;
    installing = false;
    await _run(() async => status = await admin.memoryStatus());
  }

  /// Annule l'installation en cours ; le fichier partiel est gardé pour une reprise.
  Future<void> cancel() async {
    final operation = _operation;
    if (operation == null) return;
    await _run(() => admin.cancel(operation));
  }

  /// Cherche par le sens ; une requête vide revient à la liste.
  Future<void> search(String query) async {
    if (query.trim().isEmpty) {
      hits = null;
      _changed();
      return;
    }
    await _run(() async => hits = await admin.searchMemories(query.trim()));
  }

  /// Remplace le texte d'un souvenir.
  Future<void> edit(String id, String text) => _run(() async {
    final changed = await admin.updateMemory(id, text);
    items = [for (final m in items) m.id == id ? changed : m];
    hits = null;
  });

  /// Oublie un souvenir.
  Future<void> delete(String id) => _run(() async {
    await admin.forgetMemory(id);
    items = [
      for (final m in items)
        if (m.id != id) m,
    ];
    hits = hits == null
        ? null
        : [
            for (final h in hits!)
              if (h.item.id != id) h,
          ];
    status = await admin.memoryStatus();
  });

  /// Oublie tous les souvenirs.
  Future<void> forgetAll() => _run(() async {
    await admin.forgetAllMemories();
    items = const [];
    hits = null;
    hasMore = false;
    status = await admin.memoryStatus();
  });

  /// Recalcule les souvenirs périmés avec le modèle courant.
  Future<void> reindex() => _run(() async {
    reindexed = (await admin.reindexMemories()).done;
    status = await admin.memoryStatus();
  });

  Future<void> _refreshItems() async {
    items = await admin.listMemories(limit: memoryPage);
    hasMore = items.length == memoryPage;
  }

  Future<void> _run(Future<void> Function() action) async {
    error = null;
    try {
      await action();
    } on Object catch (e) {
      error = engineErrorText(e);
    }
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
