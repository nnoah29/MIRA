import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_admin.dart';

import 'fake_ai_engine.dart';

void main() {
  late FakeAiEngine engine;
  late AiAdmin admin;
  setUp(() {
    engine = FakeAiEngine();
    admin = AiAdmin(engine);
  });

  test('parses_tool_called_and_finished_lines', () async {
    engine.scripts.add([
      {
        'type': 'tool_called',
        'id': 'c1',
        'name': 'search_messages',
        'summary': 'devis',
      },
      {
        'type': 'tool_finished',
        'id': 'c1',
        'state': 'ok',
        'results': 2,
        'sources': ['m1', 'm2'],
        'denied_kind': null,
      },
      {
        'type': 'tool_finished',
        'id': 'c2',
        'state': 'denied',
        'results': 0,
        'sources': <String>[],
        'denied_kind': 'mail_list',
      },
    ]);
    final events = await admin.chat(const {}).toList();
    final called = events[0] as AiToolCalled;
    expect(
      (called.id, called.name, called.summary),
      ('c1', 'search_messages', 'devis'),
    );
    final ok = events[1] as AiToolFinished;
    expect((ok.state, ok.results), ('ok', 2));
    expect(ok.sources, ['m1', 'm2']);
    expect(ok.deniedKind, isNull);
    final denied = events[2] as AiToolFinished;
    expect((denied.state, denied.deniedKind), ('denied', 'mail_list'));
  });

  test('request_carries_allow_tools', () async {
    engine.scripts.add(const []);
    await admin.chat({'messages': const []}).toList();
    expect(engine.chatStarts.single['allow_tools'], isTrue);
  });
}
