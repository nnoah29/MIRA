import '../data/format.dart';
import '../data/models.dart';

/// Dossier de l'interface correspondant au rôle d'une boîte du moteur (`null` : non affichée).
String? folderForRole(String role) => switch (role) {
  'inbox' => 'inbox',
  'sent' => 'sent',
  'drafts' => 'drafts',
  'trash' => 'trash',
  'junk' => 'spam',
  'archive' => 'archive',
  _ => null,
};

/// Personne d'une adresse d'en-tête `{name, address}`.
Person personOf(Object? entry) {
  final e = (entry as Map?)?.cast<String, dynamic>() ?? const {};
  final address = '${e['address'] ?? ''}';
  final name = e['name'] as String?;
  return Person(name == null || name.trim().isEmpty ? address : name, address);
}

/// Personnes d'une liste d'adresses d'en-tête.
List<Person> peopleOf(Object? entries) => [
  for (final e in (entries as List? ?? const [])) personOf(e),
];

/// Compte de l'interface à partir d'un compte du moteur.
Account accountOf(Map<String, dynamic> a) =>
    Account('${a['id']}', '${a['display_name']}', '${a['address']}');

/// Extension de fichier en majuscules (`PDF`, `PNG`…), ou le sous-type MIME.
String attachmentKind(String? filename, String mime) {
  final dot = filename?.lastIndexOf('.') ?? -1;
  if (filename != null && dot > 0 && dot < filename.length - 1) {
    return filename.substring(dot + 1).toUpperCase();
  }
  return mime.split('/').last.toUpperCase();
}

/// Pièce jointe de l'interface à partir de ses métadonnées du moteur.
Attachment attachmentOf(Map<String, dynamic> a) {
  final name = (a['filename'] as String?) ?? 'pièce jointe';
  return Attachment(
    name,
    attachmentKind(name, '${a['mime_type']}'),
    sizeOf((a['size'] as num?)?.toInt() ?? 0),
    id: a['id'] as String?,
  );
}

/// Message du moteur → message de l'interface (corps = aperçu tant qu'il n'est pas chargé).
Msg msgOf(Map<String, dynamic> m) {
  final from = peopleOf(m['from']);
  final date = (m['date'] ?? m['internal_date'] ?? 0) as num;
  final preview = '${m['preview'] ?? ''}';
  return Msg(
    from: from.isEmpty ? const Person('?', '') : from.first,
    date: date.toInt(),
    body: [
      Para([Span(preview)]),
    ],
    ref: '${m['id']}',
    hasAttachments: m['has_attachments'] == true,
    unsub: _unsubOf(m['unsubscribe']),
  );
}

Unsub? _unsubOf(Object? raw) {
  if (raw is! Map) return null;
  final url = raw['https_url'], mailto = raw['mailto'];
  if (url is! String && mailto is! String) return null;
  return Unsub(
    url: url is String ? url : null,
    mailto: mailto is String ? mailto : null,
    oneClick: raw['one_click'] == true,
  );
}
