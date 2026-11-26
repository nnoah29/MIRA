//! Fil de l'assistant écrit en texte JSON pour l'historique : le texte affiché, jamais ce qui part au modèle.

import 'dart:convert';

import '../engine/ai_models.dart';
import 'chat_message.dart';

/// Version de la forme du contenu enregistré.
const chatPayloadVersion = 1;

Map<String, dynamic> _toolJson(ToolActivity t) => {
  'id': t.id,
  'name': t.name,
  'summary': t.summary,
  'state': t.state.name,
  'results': t.results,
  'denied_kind': t.deniedKind,
};

Map<String, dynamic> _msgJson(ChatMsg m) => {
  'role': m.role,
  'text': m.text,
  'draft': m.draft,
  'refs': m.refs,
  'sources': m.sources,
  'drafts': [for (final d in m.drafts) d.toJson()],
  'actions': [
    for (final a in m.actions) {...a.proposal.toJson(), 'status': a.status},
  ],
  'memories': [
    for (final x in m.memories)
      {...x.proposal.toJson(), 'status': x.status, 'target': x.target},
  ],
  'cards': [for (final c in m.cards) c.toJson()],
  'tools': [for (final t in m.tools) _toolJson(t)],
  'failed': m.failed,
  'note': m.note,
  'masked': m.maskedCount,
  'engine': m.fromEngine,
  'attachments': [
    for (final a in m.attachments) {'kind': a.kind, 'label': a.label},
  ],
};

/// Écrit les messages (sans l'accueil) en texte JSON `{"messages": [...]}`.
String encodeChat(List<ChatMsg> messages) => jsonEncode({
  'messages': [for (final m in messages) _msgJson(m)],
});

List<String>? _strings(Object? raw) =>
    raw == null ? null : [for (final s in raw as List) s as String];

ToolActivity _tool(Map<String, dynamic> j) => ToolActivity(
  id: j['id'] as String,
  name: j['name'] as String,
  summary: j['summary'] as String? ?? '',
  state: ToolState.values.byName(j['state'] as String),
  results: (j['results'] as num?)?.toInt() ?? 0,
  deniedKind: j['denied_kind'] as String?,
);

ChatMsg _msg(Map<String, dynamic> j) {
  final role = j['role'] as String;
  if (role != 'me' && role != 'ai') throw FormatException(role);
  final text = j['text'] as String;
  final m = ChatMsg(
    role: role,
    text: text,
    shown: text,
    draft: _strings(j['draft']),
    refs: _strings(j['refs']),
    fromEngine: j['engine'] == true,
    failed: j['failed'] == true,
    note: j['note'] as String?,
    maskedCount: (j['masked'] as num?)?.toInt() ?? 0,
    attachments: [
      for (final a in (j['attachments'] as List?) ?? const [])
        ChatAttachmentRef((a as Map)['kind'] as String, a['label'] as String),
    ],
  );
  m.sources.addAll(_strings(j['sources']) ?? const []);
  for (final t in (j['tools'] as List?) ?? const []) {
    m.tools.add(_tool((t as Map).cast<String, dynamic>()));
  }
  for (final a in (j['actions'] as List?) ?? const []) {
    final json = (a as Map).cast<String, dynamic>();
    m.actions.add(
      ProposedAction(
        AiActionProposal.fromJson(json),
        status: json['status'] as String? ?? 'pending',
      ),
    );
  }
  for (final x in (j['memories'] as List?) ?? const []) {
    final json = (x as Map).cast<String, dynamic>();
    m.memories.add(
      ProposedMemory(
        AiMemoryProposal.fromJson(json),
        status: json['status'] as String? ?? 'pending',
        target: json['target'] as String?,
      ),
    );
  }
  for (final c in (j['cards'] as List?) ?? const []) {
    m.cards.add(AiMailCards.fromJson((c as Map).cast<String, dynamic>()));
  }
  for (final d in (j['drafts'] as List?) ?? const []) {
    m.drafts.add(AiDraftProposal.fromJson((d as Map).cast<String, dynamic>()));
  }
  return m;
}

/// Relit les messages ; `null` si la version est inconnue ou le texte illisible.
List<ChatMsg>? decodeChat(String payload, int version) {
  if (version != chatPayloadVersion) return null;
  try {
    final root = jsonDecode(payload) as Map<String, dynamic>;
    final items = root['messages'] as List;
    return [for (final i in items) _msg((i as Map).cast<String, dynamic>())];
  } on Object {
    return null;
  }
}
