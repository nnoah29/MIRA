import 'dart:math';

import '../data/models.dart';
import 'engine_client.dart';

/// Libellés d'un compte : boîtes de rôle « autre » du moteur, création, renommage, suppression, pose.
class LabelAdmin {
  final EngineClient client;
  int _seq = 0;
  final String _run = Random().nextInt(1 << 32).toRadixString(36);
  LabelAdmin(this.client);

  String _key(String what) =>
      'ui-$what-$_run-${DateTime.now().microsecondsSinceEpoch}-${_seq++}';

  /// Crée un libellé (une boîte racine) ; le moteur l'envoie au serveur ensuite.
  Future<void> create(String account, String name) => client.call(
    req('action', 'create_mailbox', {
      'account': account,
      'name': name,
      'parent': null,
      'idempotency_key': _key('create'),
    }),
  );

  /// Renomme un libellé.
  Future<void> rename(String id, String name) => client.call(
    req('action', 'rename_mailbox', {
      'mailbox': id,
      'name': name,
      'idempotency_key': _key('rename'),
    }),
  );

  /// Supprime un libellé (les messages restent, seul le libellé disparaît).
  Future<void> delete(String id) => client.call(
    req('action', 'delete_mailbox', {
      'mailbox': id,
      'idempotency_key': _key('delete'),
    }),
  );

  /// Pose ou retire un libellé sur les messages d'une conversation.
  Future<void> setOn(Mail mail, String id, bool on) {
    final refs = [
      for (final m in mail.msgs)
        if (m.ref != null) m.ref!,
    ];
    return client.call(
      req('action', 'apply_action', {
        'kind': {
          'type': on ? 'add_to_mailbox' : 'remove_from_mailbox',
          'mailbox': id,
        },
        'targets': {'type': 'messages', 'messages': refs},
        'idempotency_key': _key('set'),
      }),
    );
  }
}
