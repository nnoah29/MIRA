import '../data/models.dart';
import 'mappers.dart';

/// Regroupe des résumés du moteur en conversations de l'interface, pour un dossier donné.
List<Mail> mailsOfFolder(
  String folder,
  String accountId,
  List<Map<String, dynamic>> summaries, {
  Set<String> labelIds = const {},
}) {
  final groups = <String, List<Map<String, dynamic>>>{};
  for (final m in summaries) {
    final key = (m['thread'] as String?) ?? '${m['id']}';
    groups.putIfAbsent(key, () => []).add(m);
  }
  return [
    for (final e in groups.entries)
      _conversation(e.key, folder, accountId, e.value, labelIds),
  ];
}

int _dateOf(Map<String, dynamic> m) =>
    ((m['date'] ?? m['internal_date'] ?? 0) as num).toInt();

Mail _conversation(
  String id,
  String folder,
  String accountId,
  List<Map<String, dynamic>> items,
  Set<String> labelIds,
) {
  final sorted = [...items]..sort((a, b) => _dateOf(a).compareTo(_dateOf(b)));
  final last = sorted.last;
  final flags = (last['flags'] as Map?)?.cast<String, dynamic>() ?? const {};
  return Mail(
    id: '$accountId/$folder/$id',
    folder: folder,
    from: personOf((last['from'] as List?)?.firstOrNull),
    to: peopleOf(last['to']),
    cc: peopleOf(last['cc']),
    subject: '${last['subject']}'.isEmpty
        ? '(sans objet)'
        : '${last['subject']}',
    date: _dateOf(last),
    unread: sorted.any((m) => (m['flags'] as Map?)?['seen'] != true),
    starred:
        sorted.any((m) => (m['flags'] as Map?)?['flagged'] == true) ||
        flags['flagged'] == true,
    labels: _labelsOf(sorted, labelIds),
    msgs: [for (final m in sorted) msgOf(m)],
    account: accountId,
  );
}

/// Libellés d'une conversation : boîtes « libellé » où se trouve au moins un de ses messages.
List<String> _labelsOf(List<Map<String, dynamic>> items, Set<String> labelIds) {
  final found = <String>{};
  for (final m in items) {
    for (final box in (m['mailboxes'] as List? ?? const [])) {
      if (labelIds.contains('$box')) found.add('$box');
    }
  }
  return found.toList()..sort();
}
