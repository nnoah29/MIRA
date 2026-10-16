import 'mail_source.dart';

import 'dart:async';

import '../data/html_lite.dart';
import '../data/models.dart';
import 'engine_client.dart';
import 'mail_builder.dart';
import 'mappers.dart';

/// Données réelles : comptes, boîtes et messages lus dans le moteur Rust.
class EngineMailSource implements MailSource {
  final EngineClient client;
  final int pageSize;
  @override
  final List<Account> accounts;
  @override
  final Map<String, List<Mail>> boxes;
  final StreamController<String> _changes = StreamController.broadcast();
  StreamSubscription<Map<String, dynamic>>? _events;

  EngineMailSource._(this.client, this.pageSize, this.accounts, this.boxes);

  /// Lit les comptes puis la première page de chaque boîte affichée.
  static Future<EngineMailSource> load(
    EngineClient client, {
    int pageSize = 200,
  }) async {
    final raw = await client.call(req('account', 'list_accounts'));
    final list = [
      for (final a in (raw['data'] as List)) accountOf((a as Map).cast()),
    ];
    final source = EngineMailSource._(client, pageSize, list, {});
    for (final a in list) {
      source.boxes[a.id] = await source.reload(a);
    }
    source._listen();
    return source;
  }

  @override
  Stream<String> get changes => _changes.stream;

  void _listen() {
    _events = client.events().listen((e) {
      final kind = (e['kind'] as Map?)?.cast<String, dynamic>() ?? const {};
      final account = kind['account'];
      if (account is String && _isChange('${kind['type']}')) {
        _changes.add(account);
      }
    }, onError: (_) {});
  }

  static bool _isChange(String type) => const {
    'sync_finished',
    'new_message_received',
    'message_updated',
    'message_deleted',
    'mailbox_counters_updated',
    'body_available',
  }.contains(type);

  @override
  Future<List<Mail>> reload(Account account) async {
    final raw = await client.call(
      req('mailbox', 'list_mailboxes', {'account': account.id}),
    );
    final out = <Mail>[];
    final seen = <String>{};
    for (final box in (raw['data'] as List).cast<Map>()) {
      final folder = folderForRole('${box['role']}');
      if (folder == null) continue;
      final msgs = await _summaries('${box['id']}');
      final fresh = msgs.where((m) => seen.add('${m['id']}')).toList();
      out.addAll(mailsOfFolder(folder, account.id, fresh));
    }
    return out;
  }

  Future<List<Map<String, dynamic>>> _summaries(String mailbox) async {
    final raw = await client.call(
      req('message', 'list_messages', {
        'view': {'type': 'mailbox', 'mailbox': mailbox},
        'filters': {
          'unread_only': false,
          'flagged_only': false,
          'has_attachments': false,
          'from_contains': null,
          'since': null,
          'until': null,
        },
        'page': {'cursor': null, 'limit': pageSize},
      }),
    );
    final items = (raw['data'] as Map)['items'] as List;
    return [for (final m in items) (m as Map).cast<String, dynamic>()];
  }

  @override
  Future<bool> hydrate(Mail mail) async {
    var changed = false;
    for (final m in mail.msgs) {
      if (m.hydrated || m.ref == null) continue;
      m.hydrated = true;
      changed |= await _hydrateMsg(m);
    }
    return changed;
  }

  Future<bool> _hydrateMsg(Msg m) async {
    final body = await _body(m.ref!);
    var changed = body != null;
    if (body != null) m.body = body;
    if (m.hasAttachments) {
      final raw = await client.call(
        req('message', 'list_attachments', {'message': m.ref}),
      );
      m.att = [
        for (final a in (raw['data'] as List)) attachmentOf((a as Map).cast()),
      ];
      changed = true;
    }
    return changed;
  }

  Future<List<Block>?> _body(String id) async {
    try {
      final raw = await _bodyView(id);
      final data = (raw['data'] as Map).cast<String, dynamic>();
      if (data['html_available'] != true) return _plain('${data['text']}');
      final html = await client.call(
        req('message', 'get_safe_html', {'message': id}),
      );
      return parseHtml('${(html['data'] as Map)['html']}');
    } on EngineException {
      return null;
    }
  }

  Future<Map<String, dynamic>> _bodyView(String id) async {
    try {
      return await client.call(
        req('message', 'get_message_body', {'message': id}),
      );
    } on EngineException catch (e) {
      if (e.code != 'incompatible_state.body_unavailable_offline') rethrow;
      return client.call(req('message', 'fetch_message_body', {'message': id}));
    }
  }

  static List<Block> _plain(String text) => [
    for (final p in text.split(RegExp(r'\n\s*\n')))
      if (p.trim().isNotEmpty) Para([Span(p.trim())]),
  ];

  /// Arrête l'écoute des événements.
  Future<void> dispose() async {
    await _events?.cancel();
    await _changes.close();
  }
}
