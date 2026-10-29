import 'package:flutter/foundation.dart';

import '../engine/ai_admin.dart';
import '../i18n/engine_errors.dart';

/// Résultat du dernier essai d'une connexion : réussi, ou le texte traduit de l'erreur.
class AiTestOutcome {
  final bool ok;
  final String? error;
  const AiTestOutcome.success() : ok = true, error = null;
  const AiTestOutcome.failure(this.error) : ok = false;
}

/// Réglages de l'IA : fournisseurs, connexions, essais ; chaque action relit les connexions du moteur.
class AiSettingsState extends ChangeNotifier {
  final AiAdmin admin;
  List<AiProviderInfo> providers = const [];
  List<AiConnectionInfo> connections = const [];
  final Set<String> busy = {};
  final Map<String, AiTestOutcome> tests = {};
  final Map<String, List<AiModelInfo>> fetched = {};
  String? error;
  bool loaded = false;
  bool _disposed = false;

  AiSettingsState(this.admin);

  /// Modèles connus d'une connexion : relevé frais s'il y en a un, sinon liste gardée par le moteur.
  List<AiModelInfo> modelsOf(AiConnectionInfo c) =>
      fetched[c.id] ?? c.models ?? const [];

  /// Fournisseur d'une connexion (`null` : inconnu de ce moteur).
  AiProviderInfo? providerOf(AiConnectionInfo c) {
    for (final p in providers) {
      if (p.id == c.provider) return p;
    }
    return null;
  }

  /// Charge fournisseurs et connexions.
  Future<void> load() async {
    try {
      providers = await admin.providers();
      connections = await admin.connections();
      error = null;
    } on Object catch (e) {
      error = engineErrorText(e);
    }
    loaded = true;
    _changed();
  }

  Future<void> _reload() async {
    connections = await admin.connections();
  }

  Future<void> _act(String key, Future<void> Function() body) async {
    busy.add(key);
    error = null;
    _changed();
    try {
      await body();
      await _reload();
    } on Object catch (e) {
      error = engineErrorText(e);
    }
    busy.remove(key);
    _changed();
  }

  /// Ajoute une connexion ; la première devient la connexion par défaut.
  Future<void> addConnection(
    String providerId, {
    required String label,
    String? baseUrl,
  }) => _act('new', () async {
    final made = await admin.add(
      provider: providerId,
      label: label,
      baseUrl: baseUrl,
    );
    await _reload();
    if (!connections.any((c) => c.isDefault)) await admin.setDefault(made.id);
  });

  /// Range la clé saisie ; elle n'est gardée nulle part ici.
  Future<void> saveKey(String id, String key) =>
      _act(id, () async => admin.setKey(id, key.trim()));

  /// Efface la clé d'une connexion.
  Future<void> clearKey(String id) => _act(id, () async => admin.clearKey(id));

  /// Essaie la connexion et garde le verdict (réussite ou erreur traduite).
  Future<void> testConnection(String id) async {
    tests.remove(id);
    await _act(id, () async {
      try {
        await admin.test(id);
        tests[id] = const AiTestOutcome.success();
      } on Object catch (e) {
        tests[id] = AiTestOutcome.failure(engineErrorText(e));
      }
    });
  }

  /// Relève les modèles du service (depuis le cache, ou de nouveau avec [refresh]).
  Future<void> loadModels(String id, {bool refresh = false}) =>
      _act(id, () async {
        fetched[id] = await admin.models(id, refresh: refresh);
      });

  /// Change le modèle par défaut d'une connexion.
  Future<void> setModel(AiConnectionInfo c, String model) => _act(
    c.id,
    () async => admin.update(
      c.id,
      label: c.label,
      baseUrl: c.baseUrl,
      defaultModel: model.trim().isEmpty ? null : model.trim(),
    ),
  );

  /// Change libellé et adresse (l'adresse n'est envoyée que si le fournisseur la laisse libre).
  Future<void> saveAddress(AiConnectionInfo c, String label, String? baseUrl) =>
      _act(c.id, () async {
        await admin.update(
          c.id,
          label: label,
          baseUrl: baseUrl,
          defaultModel: c.defaultModel,
        );
        fetched.remove(c.id);
        tests.remove(c.id);
      });

  /// Désigne la connexion par défaut.
  Future<void> setDefault(String id) =>
      _act(id, () async => admin.setDefault(id));

  /// Supprime la connexion et sa clé.
  Future<void> remove(String id) => _act(id, () async {
    await admin.remove(id);
    fetched.remove(id);
    tests.remove(id);
  });

  void _changed() {
    if (!_disposed) notifyListeners();
  }

  @override
  void dispose() {
    _disposed = true;
    super.dispose();
  }
}
