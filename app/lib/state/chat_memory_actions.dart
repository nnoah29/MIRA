//! Propositions de mémoire du fil : retrouver le souvenir à oublier, puis appliquer la proposition confirmée.

import '../engine/ai_admin.dart';
import '../engine/ai_memory_admin.dart';
import 'chat_message.dart';

const _lookupLimit = 200;

/// Retrouve le texte du souvenir que l'IA propose d'oublier ; s'il n'existe plus, la proposition est marquée sans objet.
Future<void> resolveForgetTarget(AiAdmin ai, ProposedMemory item) async {
  if (item.proposal.action != 'forget') return;
  final wanted = item.proposal.memoryId;
  try {
    final all = await ai.listMemories(limit: _lookupLimit);
    final hit = all.where((m) => m.id == wanted);
    item.target = hit.isEmpty ? null : hit.first.text;
    item.missing = hit.isEmpty;
  } on Object {
    item.missing = false;
  }
}

/// Applique la proposition confirmée : retient le texte, ou oublie le souvenir ; une erreur remonte à l'appelant.
Future<void> applyMemoryProposal(AiAdmin ai, ProposedMemory item) async {
  final p = item.proposal;
  if (p.action == 'forget') {
    await ai.forgetMemory(p.memoryId ?? '');
  } else {
    await ai.remember(p.text, kind: p.kind);
  }
}
