import 'mail_source.dart';

import 'dart:async';
import 'dart:io';

import 'package:flutter/foundation.dart';

import '../data/format.dart';
import '../data/html_lite.dart';
import '../data/models.dart';
import 'account_admin.dart';
import 'account_setup.dart';
import 'compose_admin.dart';
import 'css_inliner.dart';
import 'inline_images.dart';
import 'label_admin.dart';
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
    this.folderMap,
  );

  /// Lit les comptes puis la première page de chaque boîte affichée.
  static Future<EngineMailSource> load(
    EngineClient client, {
    int pageSize = 200,
    Duration debounce = const Duration(milliseconds: 150),
    Map<String, String> folderMap = const {},
  }) async {
    final raw = await client.call(req('account', 'list_accounts'));
    final list = [
      for (final a in (raw['data'] as List)) accountOf((a as Map).cast()),
    ];
    final source = EngineMailSource._(client, pageSize, debounce, list, {}, {
      ...folderMap,
    });
    for (final a in list) {
      source.boxes[a.id] = await source.reload(a);
    }
    await source.refreshIdentities();
    source._listen();
    return source;
  }

  @override
  final Map<String, List<IdentityDetail>> identities = {};

  @override
  Future<void> refreshIdentities() async {
    final admin = AccountAdmin(client);
    final fresh = <String, List<IdentityDetail>>{};
    for (final a in accounts) {
      try {
        fresh[a.id] = await admin.identities(a.id);
      } on EngineException {
        fresh[a.id] = identities[a.id] ?? const [];
      }
    }
    identities
      ..clear()
      ..addAll(fresh);
  }

  @override
  AccountSetup get setup => AccountSetup(client);

  @override
  AccountAdmin get admin => AccountAdmin(client);

  @override
  final Map<String, LabelDef> labels = {};

  @override
  final Map<String, LabelDef> mappable = {};

  @override
  final Map<String, String> folderMap;

  @override
  LabelAdmin get labelAdmin => LabelAdmin(client);

  @override
  ComposeAdmin get composer => ComposeAdmin(client);

  final Map<String, Uint8List> _attachments = {};

  @override
  Future<Uint8List?> attachmentBytes(Attachment a) async {
    final id = a.id;
    if (id == null) return null;
    final known = _attachments[id];
    if (known != null) return known;
    final bytes = await fetchAttachment(client, id);
    if (bytes != null) _attachments[id] = bytes;
    return bytes;
  }

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
    await refreshIdentities();
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
    final boxes = <Map<String, dynamic>>[
      for (final b in (raw['data'] as List)) (b as Map).cast<String, dynamic>(),
    ];
    final found = _labelsOf(account, boxes);
    final byFolder = <String, List<Map<String, dynamic>>>{};
    final seen = <String>{};
    final ordered = [
      ...boxes.where((b) => !folderMap.containsKey('${b['id']}')),
      ...boxes.where((b) => folderMap.containsKey('${b['id']}')),
    ];
    for (final box in ordered) {
      final folder = _folderOf(box);
      if (folder == null) continue;
      final msgs = await _summaries('${box['id']}');
      byFolder
          .putIfAbsent(folder, () => [])
          .addAll(msgs.where((m) => seen.add('${m['id']}')));
    }
    await _addLabelOnly(byFolder, boxes, found, seen);
    final mine = await _drafts(account);
    _dropDraftCopies(byFolder, mine);
    return [
      for (final e in byFolder.entries)
        ...mailsOfFolder(
          e.key,
          account.id,
          e.value,
          labelIds: found.keys.toSet(),
        ),
      ...mine.map((d) => _draftMail(account, d)),
    ];
  }

  /// Brouillons en cours d'écriture, gardés par le moteur ; une lecture ratée n'efface rien.
  Future<List<EngineDraft>> _drafts(Account account) async {
    try {
      final list = await composer.list(account.id);
      return [
        for (final d in list)
          if (d.editing) d,
      ];
    } on EngineException {
      return const [];
    }
  }

  /// Le serveur garde une copie de chaque brouillon (même Message-ID) : on ne montre que le brouillon du moteur.
  void _dropDraftCopies(
    Map<String, List<Map<String, dynamic>>> byFolder,
    List<EngineDraft> mine,
  ) {
    final copies = {for (final d in mine) 'draft-${d.id}@mira.local'};
    byFolder['drafts']?.removeWhere(
      (m) => copies.contains(
        '${m['message_id_header']}'.replaceAll(RegExp(r'^<|>$'), ''),
      ),
    );
  }

  Mail _draftMail(Account account, EngineDraft d) {
    final sender = identities[account.id]
        ?.where((i) => i.id == d.identity)
        .firstOrNull;
    final from = Person(
      sender?.displayName ?? account.name,
      sender?.address ?? account.email,
    );
    final html = d.bodyHtml;
    final body = html != null && html.trim().isNotEmpty
        ? parseHtml(html)
        : _plain(d.bodyText);
    return Mail(
      id: 'draft:${d.id}',
      folder: 'drafts',
      account: account.id,
      from: from,
      to: d.to,
      cc: d.cc,
      subject: d.subject.trim().isEmpty ? '(sans objet)' : d.subject,
      date: d.updatedAt,
      msgs: [
        Msg(from: from, date: d.updatedAt, body: body, att: d.attachments),
      ],
      draft: DraftMeta(from: from.email, bcc: d.bcc, mode: 'draft'),
    );
  }

  /// Messages présents seulement dans des libellés (ni réception, ni envoyés…) : classés dans Archive.
  Future<void> _addLabelOnly(
    Map<String, List<Map<String, dynamic>>> byFolder,
    List<Map<String, dynamic>> boxes,
    Map<String, LabelDef> labelBoxes,
    Set<String> seen,
  ) async {
    for (final box in boxes) {
      if (!labelBoxes.containsKey('${box['id']}')) continue;
      final msgs = await _summaries('${box['id']}');
      final fresh = msgs.where((m) => seen.add('${m['id']}')).toList();
      byFolder.putIfAbsent('archive', () => []).addAll(fresh);
    }
  }

  /// Dossier d'une boîte : rattachement choisi par l'utilisateur, sinon son rôle.
  String? _folderOf(Map<String, dynamic> box) =>
      folderMap['${box['id']}'] ?? folderForRole('${box['role']}');

  static bool _mappable(Map<String, dynamic> box) =>
      const {'other', 'all', 'flagged'}.contains('${box['role']}');

  /// Libellés du compte (boîtes « autre » non rattachées) ; remplace ceux déjà connus pour ce compte.
  Map<String, LabelDef> _labelsOf(
    Account account,
    List<Map<String, dynamic>> boxes,
  ) {
    final all = {
      for (final b in boxes)
        if (_mappable(b))
          '${b['id']}': LabelDef(
            _labelName(b),
            colorOf('${b['id']}'),
            account: account.id,
            system: '${b['role']}' != 'other',
          ),
    };
    final found = {
      for (final b in boxes)
        if ('${b['role']}' == 'other' && !folderMap.containsKey('${b['id']}'))
          '${b['id']}': all['${b['id']}']!,
    };
    mappable.removeWhere((_, d) => d.account == account.id);
    mappable.addAll(all);
    labels.removeWhere((_, d) => d.account == account.id);
    labels.addAll(found);
    return found;
  }

  static String _labelName(Map<String, dynamic> box) {
    final shown = '${box['display_name'] ?? ''}'.trim();
    return shown.isEmpty ? '${box['name'] ?? box['id']}' : shown;
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
