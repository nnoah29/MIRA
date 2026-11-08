import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/platform_open.dart';

void main() {
  final url = Uri.parse('https://login.microsoftonline.com/a?x=1&y=2');
  final ran = <List<String>>[];

  Future<ProcessResult> runner(String exe, List<String> args) async {
    ran.add([exe, ...args]);
    return ProcessResult(1, 0, '', '');
  }

  setUp(ran.clear);

  test('the_system_launcher_is_used_first_and_nothing_else_runs', () async {
    expect(
      await openExternal(
        url,
        primary: (_) async => true,
        run: runner,
        windows: true,
      ),
      isTrue,
    );
    expect(ran, isEmpty);
  });

  test('on_windows_a_refusal_falls_back_to_the_url_protocol_handler', () async {
    final opened = await openExternal(
      url,
      primary: (_) async => false,
      run: runner,
      windows: true,
    );
    expect(opened, isTrue);
    expect(ran.single, [
      'rundll32',
      'url.dll,FileProtocolHandler',
      url.toString(),
    ]);
  });

  test('on_windows_a_launcher_error_also_falls_back', () async {
    final opened = await openExternal(
      url,
      primary: (_) async => throw StateError('plugin absent'),
      run: runner,
      windows: true,
    );
    expect(opened, isTrue);
    expect(ran, hasLength(1));
  });

  test('off_windows_a_refusal_stays_a_refusal', () async {
    expect(
      await openExternal(
        url,
        primary: (_) async => false,
        run: runner,
        windows: false,
      ),
      isFalse,
    );
    expect(ran, isEmpty);
  });

  test('only_web_and_mail_addresses_are_ever_opened', () async {
    for (final bad in [
      'file:///C:/Windows/system32/calc.exe',
      'javascript:alert(1)',
      'ms-settings:',
    ]) {
      expect(
        await openExternal(
          Uri.parse(bad),
          primary: (_) async => false,
          run: runner,
          windows: true,
        ),
        isFalse,
        reason: bad,
      );
    }
    expect(ran, isEmpty);
  });
}
