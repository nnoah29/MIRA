import '../data/models.dart';
import 'automated_senders.dart';
import 'identity.dart';
import 'local_labels.dart';

const _tracked = {'inbox', 'sent', 'archive'};

/// Délai sans réponse avant qu'un message envoyé passe « en attente ».
const waitingAfter = Duration(days: 2);

/// Pose ou retire `mention` et `waiting` selon le dernier message de chaque fil, sans toucher aux autres marqueurs.
void markMentionsAndWaiting(
  List<Mail> mails,
  Identity me, {
  int? now,
  Duration after = waitingAfter,
}) {
  final since =
      (now ?? DateTime.now().millisecondsSinceEpoch) - after.inMilliseconds;
  final live = mails.where((m) => _tracked.contains(m.folder)).toList();
  final last = _lastOfThreads(live);
  for (final m in live) {
    final end = last[threadKeyOf(m.id)];
    final mine = end != null && me.owns(end.$2.from.email);
    final waiting =
        end != null &&
        mine &&
        end.$2.date <= since &&
        _awaitsOthers(end.$1, me);
    final cited = end != null && !mine && _mentionedBy(end.$2, me);
    _set(m, 'waiting', waiting);
    _set(m, 'mention', cited);
  }
}

/// Dernier message de chaque fil, avec la conversation qui le porte.
Map<String, (Mail, Msg)> _lastOfThreads(List<Mail> mails) {
  final last = <String, (Mail, Msg)>{};
  for (final m in mails.where((m) => m.draft == null && m.outbox == null)) {
    for (final msg in m.msgs) {
      final key = threadKeyOf(m.id);
      if (last[key] == null || msg.date >= last[key]!.$2.date) {
        last[key] = (m, msg);
      }
    }
  }
  return last;
}

/// Le message a-t-il au moins un destinataire humain autre que moi ?
bool _awaitsOthers(Mail carrier, Identity me) => [
  ...carrier.to,
  ...carrier.cc,
].any((p) => !me.owns(p.email) && !isAutomated(p.email));

/// Un expéditeur humain me cite-t-il dans ce message ?
bool _mentionedBy(Msg msg, Identity me) =>
    !isAutomated(msg.from.email) && _mentionsMe(msg, me);

/// Le texte non cité du message contient-il mon adresse ou `@` suivi de mon nom ?
bool _mentionsMe(Msg msg, Identity me) {
  final text = msg.body
      .whereType<Para>()
      .expand((p) => p.text.split('\n'))
      .where((l) => !l.trimLeft().startsWith('>'))
      .join('\n')
      .toLowerCase();
  final named = me.names.map(
    (n) => RegExp('(^|[^\\w.@-])@${RegExp.escape(n)}(?![\\w-])'),
  );
  return me.addresses.any(text.contains) || named.any(text.contains);
}

/// Ajoute ou retire un marqueur d'une conversation.
void _set(Mail m, String flag, bool on) {
  m.flags = [...m.flags.where((f) => f != flag), if (on) flag];
}
