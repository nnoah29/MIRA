import 'dart:async';

import '../../i18n/i18n.dart';
import '../engine_client.dart';
import 'byte_channel.dart';
import 'connection.dart';
import 'endpoint.dart';
import 'event_stream.dart';
import 'frame_builders.dart';
import 'host_state.dart';

/// Client du moteur par le démon : le même contrat JSON que le pont FFI, porté par le protocole de trames.
class RemoteEngineClient implements EngineClient, ChatStreamingClient {
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
  }) async {
    final connect = channelFactory ?? endpoint.connect;
    Future<RemoteConnection> open() async => RemoteConnection.open(
      await connect(),
      token: endpoint.token,
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
    } on Object {
      throw EngineException.local(
        'transport.unavailable',
        t('Moteur en arrière-plan injoignable'),
      );
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
  Stream<Map<String, dynamic>> events({int? after}) =>
      EventPump(live, after: after, backoff: _eventBackoff).stream;

  @override
  Stream<Map<String, dynamic>> chatStream(Map<String, dynamic> start) =>
      throw UnimplementedError('flux de chat : tâche suivante');

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
