import 'dart:math';

import '../data/models.dart';
import 'engine_client.dart';

/// Actions sur les messages (lu, favori, archive, corbeille, spam, déplacement) envoyées au moteur.
class MessageActions {
  final EngineClient client;
  int _seq = 0;
  final String _run = Random().nextInt(1 << 32).toRadixString(36);
  MessageActions(this.client);

  String _key() =>
      'ui-act-$_run-${DateTime.now().microsecondsSinceEpoch}-${_seq++}';

  /// Marque comme lus ou non lus.
  Map<String, dynamic> read(bool read) => {'type': 'mark_read', 'read': read};

  /// Pose ou retire le favori.
  Map<String, dynamic> flag(bool flagged) => {
    'type': 'mark_flagged',
    'flagged': flagged,
  };

  /// Range dans les archives.
  Map<String, dynamic> get archive => {'type': 'archive'};

  /// Met à la corbeille.
  Map<String, dynamic> get trash => {'type': 'trash'};

  /// Supprime pour de bon.
  Map<String, dynamic> get deleteForever => {'type': 'delete_permanently'};

  /// Signale (ou retire le signalement) comme spam.
  Map<String, dynamic> spam(bool spam) => {'type': 'mark_spam', 'spam': spam};

  /// Déplace dans une boîte du serveur.
  Map<String, dynamic> move(String mailbox) => {'type': 'move', 'to': mailbox};

  /// Vide une boîte du serveur (corbeille, spam) d'un seul appel, sans lister ses messages.
  Future<void> emptyMailbox(String mailbox) => client.call(
    req('action', 'empty_mailbox', {
      'mailbox': mailbox,
      'idempotency_key': _key(),
    }),
  );

  /// Applique l'action à ces messages, un appel par compte ; les messages sans identifiant moteur sont ignorés.
  Future<void> run(List<Mail> mails, Map<String, dynamic> kind) async {
    final byAccount = <String, List<String>>{};
    for (final m in mails) {
      for (final x in m.msgs) {
        final ref = x.ref;
        if (ref != null) {
          byAccount.putIfAbsent(m.account, () => []).add(ref);
        }
      }
    }
    for (final refs in byAccount.values) {
      await client.call(
        req('action', 'apply_action', {
          'kind': kind,
          'targets': {'type': 'messages', 'messages': refs},
          'idempotency_key': _key(),
        }),
      );
    }
  }
}
