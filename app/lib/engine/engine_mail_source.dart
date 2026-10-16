import 'mail_source.dart';

import 'dart:async';
import 'dart:io';

import 'package:flutter/foundation.dart';

import '../data/html_lite.dart';
import '../data/models.dart';
import 'account_admin.dart';
import 'account_setup.dart';
import 'css_inliner.dart';
import 'inline_images.dart';
import 'engine_client.dart';
import 'mail_builder.dart';
import 'mappers.dart';

/// Données réelles : comptes, boîtes et messages lus dans le moteur Rust.
class EngineMailSource implements MailSource {
  final EngineClient client;
  final int pageSize;

  /// Délai de calme avant de relire une boîte après un événement.
  final Duration debounce;
  final Map<String, Timer> _pending = {};
  @override
  List<Account> accounts;
  @override
  final Map<String, List<Mail>> boxes;
  final StreamController<String> _changes = StreamController.broadcast();
  StreamSubscription<Map<String, dynamic>>? _events;

  EngineMailSource._(
    this.client,
    this.pageSize,
    this.debounce,
    this.accounts,
    this.boxes,
  );

  /// Lit les comptes puis la première page de chaque boîte affichée.
  static Future<EngineMailSource> load(
    EngineClient client, {
    int pageSize = 200,
    Duration debounce = const Duration(milliseconds: 150),
  }) async {
    final raw = await client.call(req('account', 'list_accounts'));
    final list = [
      for (final a in (raw['data'] as List)) accountOf((a as Map).cast()),
    ];
    final source = EngineMailSource._(client, pageSize, debounce, list, {});
    for (final a in list) {
      source.boxes[a.id] = await source.reload(a);
    }
    source._listen();
    return source;
  }

  @override
  AccountSetup get setup => AccountSetup(client);

  @override
  AccountAdmin get admin => AccountAdmin(client);

  @override
  Future<void> syncNow() async {
    await client.call(
      req('sync', 'start_sync', {
        'scope': {'type': 'all'},
      }),
    );
  }

  @override
  Future<List<Account>> refreshAccounts() async {
    final raw = await client.call(req('account', 'list_accounts'));
    final fresh = [
      for (final a in (raw['data'] as List)) accountOf((a as Map).cast()),
    ];
    final known = {for (final a in accounts) a.id};
    final added = fresh.where((a) => !known.contains(a.id)).toList();
    accounts = fresh;
    final ids = {for (final a in fresh) a.id};
    boxes.removeWhere((id, _) => !ids.contains(id));
    for (final a in added) {
      boxes[a.id] = await reload(a);
    }
    return added;
  }

  @override
  Stream<String> get changes => _changes.stream;

  void _listen() {
    _events = client.events().listen((e) {
      final kind = (e['kind'] as Map?)?.cast<String, dynamic>() ?? const {};
      final account = kind['account'];
      if (account is String && _isChange('${kind['type']}')) _schedule(account);
    }, onError: (_) {});
  }

  /// Regroupe les rafales d'événements d'un compte en une seule notification.
  void _schedule(String account) {
    _pending[account]?.cancel();
    _pending[account] = Timer(debounce, () {
      _pending.remove(account);
      if (!_changes.isClosed) _changes.add(account);
    });
  }

  static bool _isChange(String type) => const {
    'sync_finished',
    'new_message_received',
    'message_updated',
    'message_deleted',
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
  Future<bool> hydrate(Mail mail, {VoidCallback? onProgress}) async {
    final todo = [
      for (final m in mail.msgs)
        if (!m.hydrated && m.ref != null) m,
    ];
    for (final m in todo) {
      m.hydrated = true;
    }
    final results = await Future.wait([
      for (final m in todo) _hydrateMsg(m, onProgress),
    ]);
    return results.any((changed) => changed);
  }

  /// Corps d'abord (affiché dès qu'il est prêt), pièces jointes et images incorporées ensuite.
  Future<bool> _hydrateMsg(Msg m, VoidCallback? onProgress) async {
    final clock = Stopwatch()..start();
    try {
      final view = await _bodyView(m.ref!);
      if (view == null) return false;
      final data = (view['data'] as Map).cast<String, dynamic>();
      await _loadHtml(m);
      final text = '${data['text']}'.trim();
      if (text.isNotEmpty) {
        m.body = _plain(text);
      } else if (m.html != null) {
        m.body = parseHtml(m.html!);
      }
    } finally {
      m.loaded = true;
    }
    onProgress?.call();
    _trace('corps ${m.ref}', clock);
    await _loadAttachments(m);
    _trace('pièces jointes ${m.ref}', clock);
    return true;
  }

  void _trace(String what, Stopwatch clock) {
    if (Platform.environment['MIRA_TRACE'] == '1') {
      debugPrint('MIRA $what : ${clock.elapsedMilliseconds} ms');
    }
  }

  /// HTML d'origine (couleurs et structure gardées) ; le moteur ne dit que quelles ressources distantes restent bloquées.
  Future<void> _loadHtml(Msg m) async {
    try {
      final parts = await Future.wait([
        client.call(req('message', 'get_safe_html', {'message': m.ref})),
        client.originalHtml(m.ref!),
      ]);
      final safe = ((parts[0] as Map<String, dynamic>)['data'] as Map)
          .cast<String, dynamic>();
      m.blocked = [for (final b in (safe['blocked_resources'] as List)) '$b'];
      final html = (parts[1] as String?) ?? '${safe['html']}';
      m.html = html.trim().isEmpty ? null : await compute(inlineStyles, html);
    } on EngineException {
      m.html = null;
    }
  }

  Future<void> _loadSafeHtml(Msg m) => _loadHtml(m);

  Future<void> _loadAttachments(Msg m) async {
    final html = m.html ?? '';
    final cids = citedCids(html);
    if (!m.hasAttachments && cids.isEmpty) return;
    final raw = await client.call(
      req('message', 'list_attachments', {'message': m.ref}),
    );
    final all = [
      for (final a in (raw['data'] as List)) (a as Map).cast<String, dynamic>(),
    ];
    m.inline = await loadInlineImages(client, cids, all);
    m.att = [
      for (final a in all)
        if (!(a['inline'] == true && m.inline.containsKey(cidOf(a))))
          attachmentOf(a),
    ];
  }

  @override
  Future<bool> allowRemote(Msg msg, {bool sender = false}) async {
    final scope = sender
        ? {'type': 'sender', 'address': msg.from.email}
        : {'type': 'message', 'message': msg.ref};
    await client.call(req('message', 'allow_remote_content', {'scope': scope}));
    await _loadSafeHtml(msg);
    return true;
  }

  Future<Map<String, dynamic>?> _bodyView(String id) async {
    try {
      return await client.call(
        req('message', 'get_message_body', {'message': id}),
      );
    } on EngineException catch (e) {
      if (e.code != 'incompatible_state.body_unavailable_offline') return null;
    }
    try {
      return await client.call(
        req('message', 'fetch_message_body', {'message': id}),
      );
    } on EngineException {
      return null;
    }
  }

  static List<Block> _plain(String text) => [
    for (final p in text.split(RegExp(r'\n\s*\n')))
      if (p.trim().isNotEmpty) Para([Span(p.trim())]),
  ];

  /// Arrête l'écoute des événements.
  Future<void> dispose() async {
    for (final t in _pending.values) {
      t.cancel();
    }
    unawaited(_events?.cancel() ?? Future<void>.value());
    await _changes.close();
  }
}
