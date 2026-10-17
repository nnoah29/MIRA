import '../data/models.dart';

/// Adresse vue dans les messages d'un compte, qui n'est pas encore une identité.
class AliasHint {
  final String address;
  final int sentAs, receivedAt;
  const AliasHint(this.address, this.sentAs, this.receivedAt);

  int get score => sentAs * 3 + receivedAt;
}

/// Adresses de ce compte vues dans ses messages : expéditeur de messages envoyés, destinataire unique de messages reçus.
///
/// Un alias (autre adresse qui arrive dans la même boîte) ne se lit pas par IMAP ; on le devine donc
/// des courriels déjà synchronisés. `known` : adresses déjà connues (compte, identités), en minuscules.
List<AliasHint> aliasHints(List<Mail> mails, Set<String> known) {
  final sent = <String, int>{}, received = <String, int>{};
  for (final m in mails) {
    if (m.folder == 'sent') {
      for (final msg in m.msgs) {
        final a = msg.from.email.toLowerCase();
        if (!known.contains(a) && a.contains('@')) sent[a] = (sent[a] ?? 0) + 1;
      }
    } else if (m.folder == 'inbox' || m.folder == 'archive') {
      final to = {
        for (final p in [...m.to, ...m.cc]) p.email.toLowerCase(),
      };
      if (to.length != 1) continue;
      final a = to.first;
      if (!known.contains(a) && a.contains('@')) {
        received[a] = (received[a] ?? 0) + 1;
      }
    }
  }
  final hints = [
    for (final a in {...sent.keys, ...received.keys})
      AliasHint(a, sent[a] ?? 0, received[a] ?? 0),
  ]..sort((x, y) => y.score.compareTo(x.score));
  return hints.take(6).toList();
}
