import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/chat_message.dart';
import 'package:mira/state/chat_message_json.dart';

void main() {
  test(
    'les pièces jointes survivent à l’enregistrement de la conversation',
    () {
      final msg = ChatMsg(
        role: 'me',
        text: 'Résume',
        attachments: const [
          ChatAttachmentRef('mail', 'Alice · Devis'),
          ChatAttachmentRef('file', 'devis.txt'),
        ],
      );
      final back = decodeChat(encodeChat([msg]), chatPayloadVersion)!.single;
      expect([for (final a in back.attachments) a.kind], ['mail', 'file']);
      expect(back.attachments.first.label, 'Alice · Devis');
    },
  );

  test('une ancienne conversation sans pièces jointes se relit', () {
    final back = decodeChat(
      '{"messages":[{"role":"me","text":"Salut"}]}',
      chatPayloadVersion,
    )!.single;
    expect(back.attachments, isEmpty);
  });
}
