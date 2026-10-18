import 'dart:math';

import '../data/models.dart';
import 'engine_client.dart';

/// Brouillon tel que le moteur le garde.
class EngineDraft {
  final String id, account, identity, subject, bodyText;
  final String? bodyHtml, inReplyTo;
  final List<Person> to, cc, bcc;
  final List<Attachment> attachments;
  final int updatedAt;

  /// Seuls les brouillons en édition sont des brouillons ; les autres sont déjà partis ou programmés.
  final bool editing;
  const EngineDraft({
    required this.id,
    required this.account,
    required this.identity,
    required this.subject,
    required this.bodyText,
    required this.bodyHtml,
    required this.inReplyTo,
    required this.to,
    required this.cc,
    required this.bcc,
    required this.attachments,
    required this.updatedAt,
    required this.editing,
  });

  static List<Person> _people(Object? raw) => [
    for (final a in (raw as List? ?? const []))
      Person(
        '${(a as Map)['name'] ?? ''}'.trim().isEmpty
            ? '${a['address']}'
            : '${a['name']}',
        '${a['address']}',
      ),
  ];

  static EngineDraft fromJson(Map<String, dynamic> j) => EngineDraft(
    id: '${j['id']}',
    account: '${j['account']}',
    identity: '${j['identity']}',
    subject: '${j['subject'] ?? ''}',
    bodyText: '${j['body_text'] ?? ''}',
    bodyHtml: j['body_html'] as String?,
    inReplyTo: j['in_reply_to'] as String?,
    to: _people(j['to']),
    cc: _people(j['cc']),
    bcc: _people(j['bcc']),
    attachments: [
      for (final a in (j['attachments'] as List? ?? const []))
        Attachment(
          '${(a as Map)['filename']}',
          '${a['mime_type']}'.split('/').last.toUpperCase(),
          '${a['size']}',
        ),
    ],
    updatedAt: (j['updated_at'] as num?)?.toInt() ?? 0,
    editing: ((j['state'] as Map?)?['type'] ?? 'editing') == 'editing',
  );
}

/// Brouillons et envois : création, enregistrement, pièces jointes, envoi, annulation.
class ComposeAdmin {
  final EngineClient client;
  int _seq = 0;
  final String _run = Random().nextInt(1 << 32).toRadixString(36);
  ComposeAdmin(this.client);

  String _key(String what) =>
      'ui-$what-$_run-${DateTime.now().microsecondsSinceEpoch}-${_seq++}';

  Future<EngineDraft> _draft(Map<String, dynamic> raw) async =>
      EngineDraft.fromJson(((raw['data'] as Map)['data'] as Map).cast());

  static Map<String, dynamic> _entry(Person p) => {
    'name': p.name == p.email ? null : p.name,
    'address': p.email,
  };

  /// Crée un brouillon neuf ou issu d'un message (`mode` : new, reply, reply_all, forward).
  Future<EngineDraft> create(
    String account, {
    String? identity,
    String mode = 'new',
    String? message,
  }) async => _draft(
    await client.call(
      req('compose', 'create_draft', {
        'account': account,
        'identity': identity,
        'mode': {'type': mode, 'message': ?message},
      }),
    ),
  );

  /// Remplace destinataires, objet et corps du brouillon.
  Future<void> update(
    String draft, {
    String? identity,
    required List<Person> to,
    required List<Person> cc,
    required List<Person> bcc,
    required String subject,
    required String text,
    required String html,
  }) => client.call(
    req('compose', 'update_draft', {
      'draft': draft,
      'changes': {
        'identity': ?identity,
        'to': [for (final p in to) _entry(p)],
        'cc': [for (final p in cc) _entry(p)],
        'bcc': [for (final p in bcc) _entry(p)],
        'subject': subject,
        'body_text': text,
        'body_html': html,
      },
    }),
  );

  /// Joint un fichier local ; rend le brouillon à jour.
  Future<EngineDraft> attach(String draft, String path) async => _draft(
    await client.call(
      req('compose', 'attach_file', {'draft': draft, 'path': path}),
    ),
  );

  /// Retire la pièce jointe à cette position.
  Future<void> detach(String draft, int position) => client.call(
    req('compose', 'remove_attachment', {'draft': draft, 'position': position}),
  );

  /// Dépose une copie à jour du brouillon dans la boîte Brouillons du serveur.
  Future<void> saveCopy(String draft) => client.call(
    req('compose', 'save_draft_copy', {
      'draft': draft,
      'idempotency_key': _key('copy'),
    }),
  );

  Future<void> delete(String draft) =>
      client.call(req('compose', 'delete_draft', {'draft': draft}));

  /// Brouillons d'un compte, plus récents d'abord.
  Future<List<EngineDraft>> list(String account) async {
    final raw = await client.call(
      req('compose', 'list_drafts', {'account': account}),
    );
    final items = ((raw['data'] as Map)['data'] as List);
    return [for (final d in items) EngineDraft.fromJson((d as Map).cast())]
      ..sort((a, b) => b.updatedAt.compareTo(a.updatedAt));
  }

  /// Met l'envoi en file (après le délai d'annulation, ou à `at` en millisecondes) ; rend l'action.
  Future<String> send(String draft, {int? at}) async {
    final raw = await client.call(
      req('compose', 'send_draft', {
        'draft': draft,
        'at': at,
        'idempotency_key': _key('send'),
      }),
    );
    return '${(raw['data'] as Map)['data']}';
  }

  /// Annule un envoi encore en attente : le brouillon revient en édition.
  Future<void> cancelSend(String action) =>
      client.call(req('compose', 'cancel_send', {'action': action}));
}
