import '../engine/ai_models.dart';
import 'chat_message.dart';

/// Applique un événement d'outil à la réponse : ajoute l'appel, puis le complète avec son issue et ses sources.
void applyToolEvent(ChatMsg reply, AiChatEvent event) {
  switch (event) {
    case AiToolCalled():
      reply.tools.add(
        ToolActivity(id: event.id, name: event.name, summary: event.summary),
      );
    case AiToolFinished():
      _finish(reply, event);
    default:
      break;
  }
}

void _finish(ChatMsg reply, AiToolFinished event) {
  final known = reply.tools.where((t) => t.id == event.id);
  final tool = known.isNotEmpty ? known.first : _created(reply, event);
  tool
    ..state = _stateOf(event.state)
    ..results = event.results
    ..deniedKind = event.deniedKind;
  for (final id in event.sources) {
    if (!reply.sources.contains(id)) reply.sources.add(id);
  }
}

ToolActivity _created(ChatMsg reply, AiToolFinished event) {
  final tool = ToolActivity(id: event.id, name: '');
  reply.tools.add(tool);
  return tool;
}

ToolState _stateOf(String state) => switch (state) {
  'ok' => ToolState.ok,
  'denied' => ToolState.denied,
  _ => ToolState.error,
};
