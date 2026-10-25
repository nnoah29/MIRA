import 'dart:async';

import 'package:flutter/foundation.dart';

import '../engine/template_admin.dart';

/// Modèles de messages du moteur : liste en mémoire, relue à chaque changement annoncé.
class Templates extends ChangeNotifier {
  final TemplateAdmin admin;
  List<EngineTemplate> items = const [];
  Object? error;
  StreamSubscription<void>? _sub;
  bool _disposed = false;

  Templates(this.admin, Stream<void> changes) {
    _sub = changes.listen((_) => reload());
  }

  /// Relit la liste ; une erreur est gardée sans vider la liste déjà connue.
  Future<void> reload() async {
    try {
      final found = await admin.list();
      if (_disposed) return;
      items = found;
      error = null;
    } catch (e) {
      if (_disposed) return;
      error = e;
    }
    notifyListeners();
  }

  @override
  void dispose() {
    _disposed = true;
    _sub?.cancel();
    super.dispose();
  }
}
