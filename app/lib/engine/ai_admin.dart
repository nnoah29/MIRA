import 'ai_models.dart';
import 'engine_client.dart';

export 'ai_models.dart';

/// Connexions d'IA et discussion portées par le moteur ; la clé part une seule fois et ne revient jamais.
class AiAdmin {
  final EngineClient client;
  const AiAdmin(this.client);

  Future<Object?> _send(String op, [Map<String, dynamic>? data]) async {
    final r = await client.call(req('ai', op, data ?? const {}));
    return (r['data'] as Map)['data'];
  }

  AiConnectionInfo _one(Object? raw) =>
      AiConnectionInfo.fromJson((raw as Map).cast<String, dynamic>());

  /// Fournisseurs connus du moteur.
  Future<List<AiProviderInfo>> providers() async {
    final items = await _send('list_ai_providers') as List;
    return [
      for (final v in items)
        AiProviderInfo.fromJson((v as Map).cast<String, dynamic>()),
    ];
  }

  /// Connexions configurées.
  Future<List<AiConnectionInfo>> connections() async {
    final items = await _send('list_ai_connections') as List;
    return [for (final v in items) _one(v)];
  }

  /// Crée une connexion (sans clé, hors défaut).
  Future<AiConnectionInfo> add({
    required String provider,
    required String label,
    String? baseUrl,
  }) async => _one(
    await _send('add_ai_connection', {
      'provider': provider,
      'label': label,
      'base_url': baseUrl,
    }),
  );

  /// Change libellé, adresse et modèle par défaut d'une connexion.
  Future<AiConnectionInfo> update(
    String id, {
    required String label,
    String? baseUrl,
    String? defaultModel,
  }) async => _one(
    await _send('update_ai_connection', {
      'connection': id,
      'label': label,
      'base_url': baseUrl,
      'default_model': defaultModel,
    }),
  );

  /// Supprime une connexion et sa clé.
  Future<void> remove(String id) =>
      _send('remove_ai_connection', {'connection': id});

  /// Range la clé au trousseau du moteur ; la réponse ne la porte pas.
  Future<AiConnectionInfo> setKey(String id, String key) async =>
      _one(await _send('set_ai_api_key', {'connection': id, 'key': key}));

  /// Efface la clé d'une connexion.
  Future<AiConnectionInfo> clearKey(String id) async =>
      _one(await _send('clear_ai_api_key', {'connection': id}));

  /// Désigne la connexion par défaut.
  Future<AiConnectionInfo> setDefault(String id) async =>
      _one(await _send('set_default_ai_connection', {'connection': id}));

  /// Essaie la connexion (relevé frais des modèles) ; lève [EngineException] si elle échoue.
  Future<void> test(String id) =>
      _send('test_ai_connection', {'connection': id});

  /// Modèles du service, depuis le cache ou, avec [refresh], relevés de nouveau.
  Future<List<AiModelInfo>> models(String id, {bool refresh = false}) async {
    final r = await _send('list_ai_models', {
      'connection': id,
      'refresh': refresh,
    });
    final list = (r as Map)['models'] as List;
    return [
      for (final m in list)
        AiModelInfo.fromJson((m as Map).cast<String, dynamic>()),
    ];
  }

  /// Enregistre l'accord de l'utilisateur pour envoyer ses textes à ce service, pour les types [kinds].
  Future<AiConnectionInfo> acknowledgeConsent(
    String id, {
    List<String> kinds = const [aiKindMail],
  }) async => _one(
    await _send('acknowledge_ai_consent', {'connection': id, 'kinds': kinds}),
  );

  /// Remplace la liste des types de contenu que ce service peut recevoir.
  Future<AiConnectionInfo> setAllowedContent(
    String id,
    List<String> kinds,
  ) async => _one(
    await _send('set_ai_allowed_content', {'connection': id, 'kinds': kinds}),
  );

  /// Règle le masquage des données sensibles de cette connexion.
  Future<AiConnectionInfo> setMasking(String id, AiMasking masking) async =>
      _one(
        await _send('set_ai_masking', {
          'connection': id,
          'masking': masking.toJson(),
        }),
      );

  /// Envois récents à un service d'IA, les plus récents d'abord ; [before] reprend après cette ligne.
  Future<List<AiJournalRow>> journal({int limit = 50, String? before}) async {
    final items = await _send('list_ai_journal', {
      'limit': limit,
      'before': before,
    });
    return [
      for (final v in items as List)
        AiJournalRow.fromJson((v as Map).cast<String, dynamic>()),
    ];
  }

  /// Efface tout le journal des envois.
  Future<void> clearJournal() => _send('clear_ai_journal');

  /// Lance une génération (`ChatRequest`) ; sans [connection], la connexion par défaut sert.
  Stream<AiChatEvent> chat(Map<String, dynamic> request, {String? connection}) {
    final Object c = client;
    if (c is! ChatStreamingClient) return Stream.error(UnimplementedError());
    return c
        .chatStream({
          'connection': connection,
          'request': request,
          'allow_tools': true,
        })
        .expand(_event);
  }

  /// Annule une génération en cours.
  Future<void> cancel(String operation) async {
    await client.call(
      req('operation', 'cancel_operation', {'operation': operation}),
    );
  }

  Iterable<AiChatEvent> _event(Map<String, dynamic> line) =>
      switch (line['type']) {
        'started' => [AiStarted('${line['operation']}')],
        'text' => [AiText('${line['text']}')],
        'masked' => [AiMasked((line['count'] as num?)?.toInt() ?? 0)],
        'usage' => [
          AiUsage(
            (line['input_tokens'] as num?)?.toInt() ?? 0,
            (line['output_tokens'] as num?)?.toInt() ?? 0,
          ),
        ],
        'tool_called' => [
          AiToolCalled(
            '${line['id']}',
            '${line['name']}',
            '${line['summary'] ?? ''}',
          ),
        ],
        'tool_finished' => [_toolFinished(line)],
        'done' => [_done(line['reason'])],
        'error' => [_failed(line['error'])],
        _ => const [],
      };

  AiToolFinished _toolFinished(Map<String, dynamic> line) => AiToolFinished(
    '${line['id']}',
    '${line['state']}',
    (line['results'] as num?)?.toInt() ?? 0,
    [for (final s in (line['sources'] as List?) ?? const []) '$s'],
    line['denied_kind'] as String?,
  );

  AiDone _done(Object? reason) {
    if (reason is Map && reason['failed'] is Map) {
      final f = (reason['failed'] as Map).cast<String, dynamic>();
      return AiDone(
        'failed',
        code: f['code'] as String?,
        retryAfterSecs: (f['retry_after_secs'] as num?)?.toInt(),
      );
    }
    return AiDone('$reason');
  }

  AiFailed _failed(Object? error) {
    final e = (error as Map?)?.cast<String, dynamic>() ?? const {};
    return AiFailed('${e['code'] ?? 'internal.bug'}', '${e['message'] ?? ''}');
  }
}
