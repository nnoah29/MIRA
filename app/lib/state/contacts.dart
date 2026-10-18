import 'dart:convert';
import 'dart:math' as math;

import '../data/models.dart';

/// Un correspondant de l'historique : adresse, nom affiché, nombre d'envois et dernier envoi.
class ContactEntry {
  final String email;

  /// Nom choisi par l'utilisateur (`null` : celui vu dans les messages).
  final String? custom;
  final String seen;
  final int count, last;
  const ContactEntry({
    required this.email,
    required this.custom,
    required this.seen,
    required this.count,
    required this.last,
  });

  String get name => custom ?? seen;
  Person get person => Person(name, email);
}

/// Vrai pour une adresse technique (désabonnement, rebond, ne-pas-répondre) qui n'est pas un correspondant.
bool isAutomatedAddress(String email) {
  final at = email.lastIndexOf('@');
  if (at < 0) return false;
  final local = email.substring(0, at).toLowerCase();
  final domain = email.substring(at + 1).toLowerCase();
  if (domain.contains('unsubscribe') || domain.startsWith('bounce')) {
    return true;
  }
  if (RegExp(r'^(no-?reply|do-?not-?reply|bounces?|mailer-daemon)([-_+.=].*)?$')
      .hasMatch(local)) {
    return true;
  }
  return local.length > 40 && !local.contains(RegExp(r'[-_ ]'));
}

/// Mémoire des destinataires : ce qu'on a lu dans les messages envoyés, gardée d'une session à l'autre.
class ContactBook {
  /// Noms choisis par l'utilisateur, par adresse (en minuscules).
  final Map<String, String> names;

  /// Historique conservé : adresse → [nom vu, envois, dernier envoi].
  final Map<String, List<Object>> history;

  /// Correspondants supprimés : adresse → instant de la suppression ; seuls des messages plus récents les ramènent.
  final Map<String, int> removed;
  ContactBook(this.names, this.history, [Map<String, int>? removed])
    : removed = removed ?? {};

  /// Relit les deux tables enregistrées ; une valeur illisible est ignorée.
  static ContactBook decode(
    String? namesJson,
    String? historyJson, [
    String? removedJson,
  ]) {
    Map<String, dynamic> read(String? j) {
      try {
        return (jsonDecode(j ?? '{}') as Map).cast<String, dynamic>();
      } catch (_) {
        return {};
      }
    }

    return ContactBook(
      {for (final e in read(namesJson).entries) e.key: '${e.value}'},
      {
        for (final e in read(historyJson).entries)
          if (e.value is List && (e.value as List).length == 3)
            e.key: [
              '${(e.value as List)[0]}',
              ((e.value as List)[1] as num).toInt(),
              ((e.value as List)[2] as num).toInt(),
            ],
      },
      {
        for (final e in read(removedJson).entries)
          if (e.value is num) e.key: (e.value as num).toInt(),
      },
    );
  }

  String encodeRemoved() => jsonEncode(removed);

  /// Retire un correspondant de l'historique et de ses noms, jusqu'à un message plus récent que `now`.
  void forget(String email, int now) {
    final key = email.toLowerCase();
    history.remove(key);
    names.remove(key);
    removed[key] = now;
  }

  String encodeNames() => jsonEncode(names);
  String encodeHistory() => jsonEncode(history);

  /// Fusionne les messages envoyés avec l'historique ; rend `true` si l'historique a changé.
  bool remember(Iterable<Mail> sent, bool Function(String) mine) {
    final seen = <String, List<Object>>{};
    for (final m in sent) {
      for (final p in [...m.to, ...m.cc]) {
        final key = p.email.toLowerCase();
        if (key.isEmpty || mine(key) || isAutomatedAddress(key)) continue;
        if (m.date <= (removed[key] ?? -1)) continue;
        final cur = seen.putIfAbsent(key, () => [p.name, 0, 0]);
        cur[1] = (cur[1] as int) + 1;
        if (m.date >= (cur[2] as int)) {
          cur[2] = m.date;
          if (p.name != p.email) cur[0] = p.name;
        }
      }
    }
    var changed = false;
    for (final e in seen.entries) {
      final old = history[e.key];
      final next = old == null
          ? e.value
          : [
              e.value[0],
              math.max(old[1] as int, e.value[1] as int),
              math.max(old[2] as int, e.value[2] as int),
            ];
      if (old == null || old.join('|') != next.join('|')) {
        history[e.key] = next;
        changed = true;
      }
    }
    return changed;
  }

  /// Correspondants, du plus récent au plus ancien.
  List<ContactEntry> entries() {
    final out = [
      for (final e in history.entries)
        if (!isAutomatedAddress(e.key))
          ContactEntry(
            email: e.key,
            custom: names[e.key],
            seen: e.value[0] as String,
            count: e.value[1] as int,
            last: e.value[2] as int,
          ),
    ];
    out.sort((a, b) => b.last.compareTo(a.last));
    return out;
  }
}
