//! État du modèle d'IA local : lu dans le moteur, installation suivie par son opération, retrait, arrêt.

import 'dart:async';

import 'package:flutter/foundation.dart';

import '../engine/ai_admin.dart';
import '../engine/ai_local_admin.dart';
import '../engine/engine_client.dart';
import '../i18n/engine_errors.dart';

/// Ce que montre l'onglet « Local » ; rien n'est lancé tant que l'utilisateur n'agit pas.
class AiLocalState extends ChangeNotifier {
  final AiAdmin admin;
  final Duration pollEvery;
  AiLocalState(
    this.admin, {
    this.pollEvery = const Duration(milliseconds: 500),
  });

  LocalRuntimeStatus? status;
  String? error;
  String? installingId;
  double progress = 0;
  String? _operation;
  bool _disposed = false;

  /// Dit si l'onglet doit être montré : le moteur a répondu qu'il prend le modèle local en charge.
  bool get supported => status?.supported ?? false;

  /// Relit l'état ; lecture seule.
  Future<void> refresh() async {
    try {
      status = await admin.localStatus();
      final running = status?.installing;
      if (running != null && _operation == null) _follow(running);
    } on Object catch (e) {
      error = engineErrorText(e);
    }
    _changed();
  }

  /// Reprend le suivi d'une installation déjà en cours côté moteur.
  void _follow(LocalInstallInfo info) {
    installingId = info.modelId;
    _operation = info.operation;
    unawaited(_poll());
  }

  /// Lance l'installation du modèle puis suit son opération jusqu'à la fin.
  Future<void> install(String modelId) async {
    error = null;
    progress = 0;
    installingId = modelId;
    _changed();
    try {
      _operation = await admin.installLocal(modelId);
    } on Object catch (e) {
      error = engineErrorText(e);
      installingId = null;
      _changed();
      return;
    }
    await _poll();
  }

  /// Interroge l'opération tant qu'elle court, puis relit l'état.
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
      if (!info.running) {
        if (info.status == 'failed') error = _failure(info.code);
        break;
      }
      _changed();
      await Future<void>.delayed(pollEvery);
    }
    _operation = null;
    installingId = null;
    await refresh();
  }

  /// Texte d'un échec d'installation, d'après le code du moteur.
  String _failure(String? code) =>
      engineErrorText(EngineException(code ?? 'internal.bug', ''));

  /// Annule l'installation en cours ; le fichier partiel est gardé pour une reprise.
  Future<void> cancel() async {
    final operation = _operation;
    if (operation == null) return;
    try {
      await admin.cancel(operation);
    } on Object catch (e) {
      error = engineErrorText(e);
      _changed();
    }
  }

  /// Supprime un modèle installé.
  Future<void> remove(String modelId) => _run(() => admin.removeLocal(modelId));

  /// Arrête le serveur pour rendre la mémoire.
  Future<void> stop() => _run(admin.stopLocal);

  Future<void> _run(Future<void> Function() action) async {
    error = null;
    try {
      await action();
    } on Object catch (e) {
      error = engineErrorText(e);
    }
    await refresh();
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
