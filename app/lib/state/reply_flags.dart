import '../data/models.dart';
import 'local_labels.dart';

/// Adresses et noms sous lesquels on peut me joindre ou me citer, en minuscules.
class Identity {
  final Set<String> addresses, names;
  const Identity(this.addresses, this.names);

  /// Identité à partir d'adresses et de noms d'affichage ; le prénom (3 lettres et plus) compte aussi.
  factory Identity.from(Iterable<String> addresses, Iterable<String> names) {
    final shown = names
        .map((n) => n.trim().toLowerCase())
        .where((n) => n.isNotEmpty);
    return Identity(
      {for (final a in addresses) a.toLowerCase()},
      {
        for (final n in shown) ...[
          n,
          if (n.split(' ').first.length > 2) n.split(' ').first,
        ],
      },
    );
  }

  /// Cette adresse est-elle l'une des miennes ?
  bool owns(String email) => addresses.contains(email.toLowerCase());
}

const _tracked = {'inbox', 'sent', 'archive'};

/// Pose ou retire `mention` et `waiting` selon le dernier message de chaque fil, sans toucher aux autres marqueurs.
void markMentionsAndWaiting(List<Mail> mails, Identity me) {
  final live = mails.where((m) => _tracked.contains(m.folder)).toList();
  final last = _lastOfThreads(live);
  for (final m in live) {
    final end = last[threadKeyOf(m.id)];
    final mine = end != null && me.owns(end.$2.from.email);
    _set(m, 'waiting', mine && _awaitsOthers(end.$1, me));
    _set(m, 'mention', end != null && !mine && _mentionsMe(end.$2, me));
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

/// Le message a-t-il au moins un destinataire autre que moi ?
bool _awaitsOthers(Mail carrier, Identity me) =>
    [...carrier.to, ...carrier.cc].any((p) => !me.owns(p.email));

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
