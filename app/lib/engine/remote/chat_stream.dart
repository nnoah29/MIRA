import 'dart:async';

import '../engine_client.dart';
import 'connection.dart';
import 'frame_builders.dart';

/// Fenêtre de crédits d'un chat, et crédits rendus à chaque palier.
const int chatWindow = 64;
const int chatRefill = 32;

/// Génération d'IA par le démon : une ligne JSON par morceau ; annuler l'abonnement envoie `cancel`.
/// Une coupure remet [lost] comme erreur du flux ; un chat n'est jamais rouvert automatiquement.
Stream<Map<String, dynamic>> remoteChatStream(
  Future<RemoteConnection> Function() live,
  EngineException Function() lost,
  Map<String, dynamic> start,
) {
  late final StreamController<Map<String, dynamic>> controller;
  RemoteConnection? connection;
  StreamSubscription<Map<String, dynamic>>? frames;
  var cancelled = false;
  var id = 0;
  var sinceRefill = 0;

  void fail(Object error) {
    if (!controller.isClosed) controller.addError(error);
    unawaited(controller.close());
  }

  void onFrame(Map<String, dynamic> frame) {
    if (cancelled) return;
    if (frame['type'] == 'chat_delta') {
      controller.add(frame['line'] as Map<String, dynamic>);
      if (++sinceRefill >= chatRefill) {
        sinceRefill = 0;
        connection?.send(creditFrame(id, chatRefill));
      }
    } else if (frame['type'] == 'end') {
      final reason = frame['reason'] as Map<String, dynamic>;
      if (reason['type'] == 'error') {
        fail(
          EngineException(
            '${reason['code'] ?? 'internal.bug'}',
            '${reason['message'] ?? 'erreur du moteur'}', // i18n:ignore
          ),
        );
      } else {
        unawaited(controller.close());
      }
    }
  }

  Future<void> open() async {
    try {
      final opened = await live();
      if (cancelled) return;
      connection = opened;
      id = opened.nextId();
      frames = opened
          .openStream(openChatFrame(id, start, credit: chatWindow))
          .listen(
            onFrame,
            onError: (Object error) =>
                fail(error is ConnectionLost ? lost() : error),
            onDone: () => unawaited(controller.close()),
          );
    } on EngineException catch (error) {
      fail(error);
    }
  }

  controller = StreamController<Map<String, dynamic>>(
    onListen: () => unawaited(open()),
    onCancel: () async {
      cancelled = true;
      if (id != 0) connection?.send(cancelFrame(id));
      await frames?.cancel();
    },
  );
  return controller.stream;
}
