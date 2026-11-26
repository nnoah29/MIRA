import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/chat_attached_file.dart';
import 'package:mira/state/chat_file_reader.dart';

Uint8List bytes(String s) => Uint8List.fromList(s.codeUnits);

void main() {
  test('un fichier texte est lu tel quel', () async {
    final f = await readChatFile('1', 'notes.txt', bytes('Bonjour'));
    expect(f.name, 'notes.txt');
    expect(f.text, 'Bonjour');
  });

  test('du texte accentué en UTF-8 est lu', () async {
    final data = Uint8List.fromList(utf8.encode('é à ü'));
    expect(plainTextOf(data), 'é à ü');
  });

  test('un contenu binaire est refusé', () async {
    final data = Uint8List.fromList([0x89, 0x50, 0x4E, 0x47, 0, 1, 2]);
    expect(
      () => readChatFile('1', 'a.bin', data),
      throwsA(
        isA<ChatFileException>().having(
          (e) => e.problem,
          'problem',
          ChatFileProblem.unreadable,
        ),
      ),
    );
  });

  test('un fichier vide ou blanc est refusé', () async {
    expect(
      () => readChatFile('1', 'a.txt', bytes('  \n ')),
      throwsA(
        isA<ChatFileException>().having(
          (e) => e.problem,
          'problem',
          ChatFileProblem.empty,
        ),
      ),
    );
  });

  test('un fichier trop gros est refusé avant lecture', () async {
    final big = Uint8List(chatFileMaxBytes + 1);
    expect(
      () => readChatFile('1', 'a.txt', big),
      throwsA(
        isA<ChatFileException>().having(
          (e) => e.problem,
          'problem',
          ChatFileProblem.tooLarge,
        ),
      ),
    );
  });

  test('un PDF se reconnaît à son nom ou à son en-tête', () {
    expect(looksLikePdf('A.PDF', bytes('x')), isTrue);
    expect(looksLikePdf('a.bin', bytes('%PDF-1.7 ...')), isTrue);
    expect(looksLikePdf('a.txt', bytes('Bonjour')), isFalse);
  });
}
