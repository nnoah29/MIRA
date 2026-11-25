import 'package:flutter_test/flutter_test.dart';
import 'package:mira/engine/ai_privacy_models.dart';
import 'package:mira/state/chat_attached_file.dart';
import 'package:mira/state/chat_context.dart';
import 'package:mira/state/chat_message.dart';

const file = ChatAttachedFile(id: '1', name: 'devis.txt', text: 'Total : 42');

void main() {
  test('un fichier joint devient une partie de contexte attachment', () {
    final part = chatFilePart(file);
    expect(part['kind'], 'attachment');
    expect(part['label'], 'devis.txt');
    expect(part['text'], 'Total : 42');
  });

  test('le texte du fichier est clippé au budget', () {
    final big = ChatAttachedFile(
      id: '2',
      name: 'gros.txt',
      text: 'x' * (chatBodyBudget + 500),
    );
    expect(chatFilePart(big)['text']!.length, chatBodyBudget + 1);
  });

  test('mails et fichiers partent ensemble dans le contexte', () {
    final request = buildChatStart(
      history: [ChatMsg(role: 'me', text: 'Bonjour')],
      input: const ChatContextInput(subject: 's', body: 'b', files: [file]),
    );
    final kinds = [for (final p in request['context'] as List) p['kind']];
    expect(kinds, ['mail_message', 'attachment']);
  });

  test('les types annoncés suivent le contenu joint', () {
    expect(const ChatContextInput().kinds, isEmpty);
    expect(const ChatContextInput(files: [file]).kinds, [aiKindAttachment]);
    expect(const ChatContextInput(subject: 's', files: [file]).kinds, [
      aiKindMail,
      aiKindAttachment,
    ]);
  });

  test('un contexte réduit aux types autorisés retire le reste', () {
    const input = ChatContextInput(subject: 's', body: 'b', files: [file]);
    expect(input.restrictedTo([aiKindMail]).files, isEmpty);
    expect(input.restrictedTo([aiKindMail]).hasMail, isTrue);
    expect(input.restrictedTo([aiKindAttachment]).hasMail, isFalse);
    expect(input.restrictedTo([aiKindAttachment]).files, hasLength(1));
    expect(input.restrictedTo(const []).kinds, isEmpty);
  });
}
