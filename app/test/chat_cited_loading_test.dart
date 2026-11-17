import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_mail_source.dart';
import 'package:mira/state/store.dart';
import 'package:shared_preferences/shared_preferences.dart';

import 'engine_source_test.dart' show FakeEngine;

Future<MailStore> startStore() async {
  SharedPreferences.setMockInitialValues({});
  final prefs = await SharedPreferences.getInstance();
  return MailStore(prefs, source: await EngineMailSource.load(FakeEngine()));
}

void main() {
  test(
    'a_cited_message_missing_from_the_lists_is_loaded_from_the_engine',
    () async {
      final store = await startStore();
      expect(store.mailOfCited('m60'), isNull);
      await store.loadCited(['m60']);
      expect(store.mailOfCited('m60')?.subject, 'Corps ancien');
      store.dispose();
    },
  );

  test('loading_cited_messages_notifies_listeners_once_per_batch', () async {
    final store = await startStore();
    var calls = 0;
    store.addListener(() => calls++);
    await store.loadCited(['m60']);
    expect(calls, 1);
    await store.loadCited(['m60']);
    expect(calls, 1);
    store.dispose();
  });

  test('the_engine_message_id_is_not_the_conversation_id', () async {
    final store = await startStore();
    await store.loadCited(['m60']);
    expect(store.byId('m60'), isNull);
    expect(store.mailOfCited('m60')?.id, 'acc1/archive/m60');
    final inbox = store.mails.first;
    final ref = inbox.msgs.first.ref!;
    expect(store.mailOfCited(ref)?.id, inbox.id);
    store.dispose();
  });
}
