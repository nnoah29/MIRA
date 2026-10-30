import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_admin.dart';
import 'package:mira/engine/engine_client.dart';

import 'fake_ai_engine.dart';

void main() {
  late FakeAiEngine engine;
  late AiAdmin admin;
  setUp(() {
    engine = FakeAiEngine();
    admin = AiAdmin(engine);
  });

  test('add_sends_domain_ai_op_add_ai_connection', () async {
    final made = await admin.add(
      provider: 'generic',
      label: 'Mon service',
      baseUrl: 'https://ia.example.org/v1',
    );
    final sent = engine.calls.single;
    expect(sent['domain'], 'ai');
    expect((sent['request'] as Map)['op'], 'add_ai_connection');
    expect(sent['request'], containsPair('provider', 'generic'));
    expect(sent['request'], containsPair('label', 'Mon service'));
    expect(
      sent['request'],
      containsPair('base_url', 'https://ia.example.org/v1'),
    );
    expect(made.provider, 'generic');
    expect(made.hasKey, isFalse);
  });

  test('set_key_sends_key_once_and_result_has_no_key', () async {
    final made = await admin.add(provider: 'generic', label: 'x');
    final saved = await admin.setKey(made.id, 'sk-secret');
    expect(engine.requestsOf('set_ai_api_key').single['key'], 'sk-secret');
    expect(engine.requestsOf('set_ai_api_key').single['connection'], made.id);
    expect(saved.hasKey, isTrue);
    expect(saved.toString().contains('sk-secret'), isFalse);
    await admin.connections();
    expect(engine.calls.where((c) => '$c'.contains('sk-secret')), hasLength(1));
  });

  test('connections_parse_has_key_and_models', () async {
    engine.connections.add({
      'id': 'ai-9',
      'provider': 'openai',
      'label': 'OpenAI',
      'base_url': null,
      'default_model': 'm1',
      'is_default': true,
      'has_key': true,
      'consent_at': 5,
      'models': {
        'models': [
          {'id': 'm1', 'created_at': 10},
          {'id': 'm2', 'created_at': null},
        ],
        'fetched_at': 3,
      },
    });
    final c = (await admin.connections()).single;
    expect(c.hasKey, isTrue);
    expect(c.isDefault, isTrue);
    expect(c.consentGiven, isTrue);
    expect(c.defaultModel, 'm1');
    expect(c.models!.map((m) => m.id), ['m1', 'm2']);
    expect(c.models!.first.createdAt, 10);
    expect(c.models!.last.createdAt, isNull);
  });

  test('providers_parse_descriptor_fields', () async {
    engine.providers.add({
      'id': 'generic',
      'display_name': 'Autre',
      'base_url': 'https://x/v1',
      'base_url_editable': true,
    });
    final p = (await admin.providers()).single;
    expect(
      [p.id, p.name, p.baseUrl, p.editableUrl],
      ['generic', 'Autre', 'https://x/v1', true],
    );
  });

  test('chat_maps_lines_to_events_in_order', () async {
    engine.scripts.add([
      {'type': 'started', 'operation': 'op-1'},
      {'type': 'text', 'text': 'Bon'},
      {'type': 'usage', 'input_tokens': 12, 'output_tokens': 5},
      {'type': 'done', 'reason': 'end_turn'},
    ]);
    final events = await admin.chat({
      'messages': const [],
    }, connection: 'ai-1').toList();
    expect(events[0], isA<AiStarted>());
    expect((events[0] as AiStarted).operation, 'op-1');
    expect((events[1] as AiText).text, 'Bon');
    expect((events[2] as AiUsage).input, 12);
    expect((events[2] as AiUsage).output, 5);
    expect((events[3] as AiDone).reason, 'end_turn');
    expect((events[3] as AiDone).failed, isFalse);
    expect(engine.chatStarts.single['connection'], 'ai-1');
    expect(engine.chatStarts.single['request'], {'messages': const []});
  });

  test('chat_failed_done_keeps_code_and_retry_delay', () async {
    engine.scripts.add([
      {
        'type': 'done',
        'reason': {
          'failed': {'code': 'server.ai_rate_limited', 'retry_after_secs': 7},
        },
      },
    ]);
    final done = (await admin.chat(const {}).single) as AiDone;
    expect(done.failed, isTrue);
    expect(done.code, 'server.ai_rate_limited');
    expect(done.retryAfterSecs, 7);
  });

  test('chat_error_line_becomes_AiFailed_with_code', () async {
    engine.scripts.add([
      {
        'type': 'error',
        'error': {
          'code': 'incompatible_state.ai_consent_required',
          'message': 'consentement requis',
        },
      },
    ]);
    final failed = (await admin.chat(const {}).single) as AiFailed;
    expect(failed.code, 'incompatible_state.ai_consent_required');
    expect(failed.message, 'consentement requis');
  });

  test('unknown_line_types_are_ignored', () async {
    engine.scripts.add([
      {'type': 'ping'},
      {'type': 'text', 'text': 'A'},
      {'type': 'future_thing', 'x': 1},
      {'type': 'done', 'reason': 'end_turn'},
    ]);
    final events = await admin.chat(const {}).toList();
    expect(events.map((e) => e.runtimeType), [AiText, AiDone]);
  });

  test('cancel_sends_operation_cancel', () async {
    await admin.cancel('op-7');
    final sent = engine.calls.single;
    expect(sent['domain'], 'operation');
    expect(sent['request'], {'op': 'cancel_operation', 'operation': 'op-7'});
  });

  test('remove_and_set_default_send_expected_ops', () async {
    final a = await admin.add(provider: 'generic', label: 'a');
    await admin.setDefault(a.id);
    await admin.test(a.id);
    await admin.models(a.id, refresh: true);
    await admin.acknowledgeConsent(a.id);
    await admin.clearKey(a.id);
    await admin.remove(a.id);
    final ops = [for (final c in engine.calls) (c['request'] as Map)['op']];
    expect(ops, [
      'add_ai_connection',
      'set_default_ai_connection',
      'test_ai_connection',
      'list_ai_models',
      'acknowledge_ai_consent',
      'clear_ai_api_key',
      'remove_ai_connection',
    ]);
    expect(engine.requestsOf('list_ai_models').single['refresh'], isTrue);
    expect(engine.connections, isEmpty);
  });

  test('engine_exception_propagates_code', () async {
    engine.failNext = const EngineException(
      'authentication.ai_key_rejected',
      'clé refusée',
    );
    expect(
      admin.test('ai-1'),
      throwsA(
        isA<EngineException>().having(
          (e) => e.code,
          'code',
          'authentication.ai_key_rejected',
        ),
      ),
    );
  });

  test('engine_without_streaming_fails_the_chat_stream', () {
    final stream = AiAdmin(_Plain()).chat(const {});
    expect(stream, emitsError(isA<UnimplementedError>()));
  });

  test('connection_parses_allowed_kinds_and_masking', () async {
    final made = await admin.add(provider: 'generic', label: 'x');
    final saved = await admin.setAllowedContent(made.id, ['mail_message']);
    expect(saved.allowedKinds, ['mail_message']);
    final masked = await admin.setMasking(
      made.id,
      const AiMasking('off', ['iban', 'email']),
    );
    expect(masked.masking.mode, 'off');
    expect(masked.masking.categories, ['iban', 'email']);
    final sent = engine.requestsOf('set_ai_masking').single['masking'] as Map;
    expect(sent, {
      'mode': 'off',
      'categories': ['iban', 'email'],
    });
  });

  test('acknowledge_consent_sends_the_ticked_kinds', () async {
    final made = await admin.add(provider: 'generic', label: 'x');
    await admin.acknowledgeConsent(made.id, kinds: const []);
    expect(
      engine.requestsOf('acknowledge_ai_consent').single['kinds'],
      isEmpty,
    );
  });

  test('journal_pages_and_clear', () async {
    engine.journal.addAll([
      for (var i = 0; i < 3; i++)
        {
          'id': 'j$i',
          'at': 1000 - i,
          'provider': 'openai',
          'model': 'm',
          'local': i == 1,
          'kinds': ['mail_message'],
          'chars': 10,
          'masked': 0,
          'input_tokens': null,
          'output_tokens': null,
          'outcome': 'end_turn',
        },
    ]);
    final first = await admin.journal(limit: 2);
    expect(first.map((r) => r.id), ['j0', 'j1']);
    expect(first[1].local, isTrue);
    final next = await admin.journal(limit: 2, before: 'j1');
    expect(next.map((r) => r.id), ['j2']);
    expect(engine.requestsOf('list_ai_journal').last['before'], 'j1');
    await admin.clearJournal();
    expect(engine.requestsOf('clear_ai_journal'), hasLength(1));
    expect(await admin.journal(), isEmpty);
  });
}

class _Plain implements EngineClient {
  @override
  Future<Map<String, dynamic>> call(Map<String, dynamic> request) async => {};
  @override
  Stream<Map<String, dynamic>> events({int? after}) => const Stream.empty();
  @override
  Future<String?> originalHtml(String messageId) async => null;
  @override
  Future<void> close() async {}
}
