import '../data/models.dart';
import 'mappers.dart';

/// Regroupe des résumés du moteur en conversations de l'interface, pour un dossier donné.
List<Mail> mailsOfFolder(
  String folder,
  String accountId,
  List<Map<String, dynamic>> summaries,
) {
  final groups = <String, List<Map<String, dynamic>>>{};
  for (final m in summaries) {
    final key = (m['thread'] as String?) ?? '${m['id']}';
    groups.putIfAbsent(key, () => []).add(m);
  }
  return [
    for (final e in groups.entries)
      _conversation(e.key, folder, accountId, e.value),
  ];
}

int _dateOf(Map<String, dynamic> m) =>
    ((m['date'] ?? m['internal_date'] ?? 0) as num).toInt();

Mail _conversation(
  String id,
  String folder,
  String accountId,
  List<Map<String, dynamic>> items,
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
    msgs: [for (final m in sorted) msgOf(m)],
    account: accountId,
  );
}
