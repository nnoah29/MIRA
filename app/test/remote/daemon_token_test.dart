import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/engine_client.dart';
import 'package:mira/engine/remote/daemon_token.dart';

final _good = 'ab12' * 16;

/// Écrit un fichier de jeton avec ce contenu et ces droits (chmod).
File tokenFile(Directory dir, String content, String mode) {
  final file = File('${dir.path}/daemon.token')..writeAsStringSync(content);
  Process.runSync('chmod', [mode, file.path]);
  return file;
}

Matcher failsWith(String code) =>
    isA<EngineException>().having((e) => e.code, 'code', code);

void main() {
  late Directory dir;
  setUp(() => dir = Directory.systemTemp.createTempSync('mira-token'));
  tearDown(() => dir.deleteSync(recursive: true));

  test('a_private_valid_token_is_read_and_trimmed', () async {
    final file = tokenFile(dir, '$_good\n', '600');
    expect(await readDaemonToken(file.path), _good);
  }, testOn: 'linux');

  test('a_missing_file_raises_token_missing', () async {
    await expectLater(
      readDaemonToken('${dir.path}/absent'),
      throwsA(failsWith('transport.token_missing')),
    );
  });

  test('a_group_readable_file_raises_token_unsafe', () async {
    final file = tokenFile(dir, _good, '640');
    await expectLater(
      readDaemonToken(file.path),
      throwsA(failsWith('transport.token_unsafe')),
    );
  }, testOn: 'linux');

  test('a_malformed_content_raises_token_invalid_without_echoing_it', () async {
    final file = tokenFile(dir, 'SECRET-CONTENT-NOT-HEX', '600');
    Object? error;
    try {
      await readDaemonToken(file.path);
    } on Object catch (e) {
      error = e;
    }
    expect(error, failsWith('transport.token_invalid'));
    expect('$error', isNot(contains('SECRET-CONTENT')));
    expect(
      (error! as EngineException).message,
      isNot(contains('SECRET-CONTENT')),
    );
  }, testOn: 'linux');
}
