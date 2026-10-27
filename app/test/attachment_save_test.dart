import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/data/attachment_save.dart';
import 'package:mira/data/models.dart';

Attachment att(String name, {String? id}) =>
    Attachment(name, 'PDF', '1 Ko', id: id ?? name);

Future<Uint8List?> bytesOf(Attachment a) async =>
    a.n == 'cassee.pdf' ? null : Uint8List.fromList(a.n.codeUnits);

void main() {
  late Directory dir;
  setUp(() => dir = Directory.systemTemp.createTempSync('mira-save'));
  tearDown(() => dir.deleteSync(recursive: true));

  test('chaque pièce est écrite sous son nom', () async {
    final report = await saveAttachments(
      [att('a.pdf'), att('b.png')],
      dir,
      bytesOf,
    );
    expect(report.saved, ['a.pdf', 'b.png']);
    expect(report.failed, isEmpty);
    expect(File('${dir.path}/a.pdf').readAsStringSync(), 'a.pdf');
  });

  test('un nom déjà pris reçoit un numéro, rien n est écrasé', () async {
    File('${dir.path}/a.pdf').writeAsStringSync('ancien');
    final report = await saveAttachments(
      [att('a.pdf'), att('a.pdf')],
      dir,
      bytesOf,
    );
    expect(report.saved, ['a (2).pdf', 'a (3).pdf']);
    expect(File('${dir.path}/a.pdf').readAsStringSync(), 'ancien');
  });

  test('une pièce illisible est signalée sans arrêter les autres', () async {
    final report = await saveAttachments(
      [att('cassee.pdf'), att('b.pdf')],
      dir,
      bytesOf,
    );
    expect(report.saved, ['b.pdf']);
    expect(report.failed, ['cassee.pdf']);
  });

  test('un nom avec des séparateurs ne sort pas du dossier', () async {
    final report = await saveAttachments(
      [att('../../evil.txt'), att('..')],
      dir,
      bytesOf,
    );
    expect(report.saved, ['evil.txt', 'pièce jointe']);
    expect(dir.listSync().length, 2);
  });
}
