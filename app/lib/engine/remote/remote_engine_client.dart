import 'dart:async';

import '../../i18n/i18n.dart';
import '../engine_client.dart';
import 'byte_channel.dart';
import 'chat_stream.dart';
import 'connection.dart';
import 'endpoint.dart';
import 'event_stream.dart';
import 'frame_builders.dart';
import 'frames.dart';
import 'host_errors.dart';
import 'host_state.dart';

/// Client du moteur par le démon : le même contrat JSON que le pont FFI, porté par le protocole de trames.
class RemoteEngineClient
    implements EngineClient, ChatStreamingClient, PresenceClient {
  RemoteEngineClient._(this._open, this._connection, this._eventBackoff);

  final Future<RemoteConnection> Function() _open;
  RemoteConnection _connection;
  Future<RemoteConnection>? _reconnecting;
  final Duration _eventBackoff;
  bool _detached = false;

  /// Se connecte au démon et fait la poignée de main ; les échecs de connexion ou de refus sont remontés tels quels.
  static Future<RemoteEngineClient> attach(
    DaemonEndpoint endpoint, {
    required String clientId,
    String clientName = 'mira-app',
    Future<ByteChannel> Function()? channelFactory,
    Duration pingInterval = const Duration(seconds: 15),
    Duration helloTimeout = const Duration(seconds: 5),
    Duration eventBackoff = const Duration(milliseconds: 500),
    Future<String> Function()? tokenProvider,
  }) async {
    final connect = channelFactory ?? endpoint.connect;
    Future<RemoteConnection> open() async => RemoteConnection.open(
      await connect(),
      token: await (tokenProvider?.call() ?? Future.value(endpoint.token)),
      clientId: clientId,
      clientName: clientName,
      pingInterval: pingInterval,
      helloTimeout: helloTimeout,
    );
    return RemoteEngineClient._(open, await open(), eventBackoff);
  }

  /// État de l'hôte annoncé à la dernière poignée de main.
  HostState get hostState => _connection.hostState;

  /// Connexion utilisable : la courante, ou une seule reconnexion partagée si elle est tombée.
  Future<RemoteConnection> live() {
    if (_detached) return Future.error(_detachedError());
    if (!_connection.isClosed) return Future.value(_connection);
    return _reconnecting ??= _reopen().whenComplete(() => _reconnecting = null);
  }

  Future<RemoteConnection> _reopen() async {
    final RemoteConnection fresh;
    try {
      fresh = await _open();
    } on HandshakeRefused catch (refusal) {
      throw refusalError(refusal);
    } on EngineException {
      rethrow;
    } on Object {
      throw unavailableError();
    }
    if (_detached) {
      await fresh.close();
      throw _detachedError();
    }
    return _connection = fresh;
  }

  EngineException _detachedError() => EngineException.local(
    'transport.closed',
    t('Le moteur en arrière-plan est détaché'),
  );

  /// Erreur rendue aux appels en vol quand la connexion tombe.
  EngineException lostError() => EngineException.local(
    'transport.disconnected',
    t('Connexion au moteur en arrière-plan perdue'),
  );

  /// Envoie une trame d'échange et rend la trame `result` ; une coupure devient `transport.disconnected`.
  Future<Map<String, dynamic>> _exchange(
    Map<String, dynamic> Function(int id) frame,
  ) async {
    final connection = await live();
    try {
      return await connection.request(frame(connection.nextId()));
    } on ConnectionLost {
      throw lostError();
    } on ProtocolException {
      throw EngineException.local(
        'transport.request_too_large',
        t('Requête trop volumineuse'),
      );
    }
  }

  @override
  Future<Map<String, dynamic>> call(Map<String, dynamic> request) async {
    final result = await _exchange((id) => callFrame(id, request));
    return openEnvelopeMap(result['envelope'] as Map<String, dynamic>);
  }

  @override
  Future<String?> originalHtml(String messageId) async {
    final result = await _exchange((id) => getOriginalHtmlFrame(id, messageId));
    final data = openEnvelopeMap(result['envelope'] as Map<String, dynamic>);
    return data['html'] as String?;
  }

  @override
  Stream<Map<String, dynamic>> events({int? after}) => EventPump(
    live,
    after: after,
    backoff: _eventBackoff,
    baseline: after == null ? latestEventSeq : null,
  ).stream;

  /// Numéro du dernier événement du journal (0 si vide), lu par pages : point de départ d'un flux sans `after`.
  Future<int> latestEventSeq() async {
    int? cursor;
    while (true) {
      final page = await call(
        req('event', 'get_events', {
          'filter': {
            'accounts': <String>[],
            'categories': <String>[],
            'after': cursor,
          },
          'limit': 500,
        }),
      );
      final events = (page['data'] as List).cast<Map<String, dynamic>>();
      if (events.isEmpty) return cursor ?? 0;
      cursor = events.last['seq'] as int;
      if (events.length < 500) return cursor;
    }
  }

  @override
  Stream<Map<String, dynamic>> chatStream(Map<String, dynamic> start) =>
      remoteChatStream(live, lostError, start);

  /// Déclare au démon si la fenêtre de l'app est au premier plan (il notifie ou non en conséquence).
  @override
  Future<void> setPresence(bool focused) async {
    await _exchange((id) => setPresenceFrame(id, focused));
  }

  /// Demande au démon de réessayer tout de suite d'ouvrir le trousseau du système.
  Future<void> retryUnlock() async {
    await _exchange(hostOpRetryUnlockFrame);
  }

  /// Détache le client : la connexion se ferme, le démon et son moteur continuent.
  @override
  Future<void> close() async {
    _detached = true;
    await _connection.close();
  }

  /// Arrête le démon (`host_op shutdown`) puis détache le client.
  Future<void> shutdown() async {
    try {
      await _exchange(hostOpShutdownFrame);
    } on EngineException {
      // le démon s'est arrêté avant de répondre : l'objectif est atteint
    }
    await close();
  }
}
