import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/attachment_mention.dart';

void main() {
  group('un texte qui annonce une pièce jointe', () {
    test('en français', () {
      for (final s in [
        'Vous trouverez ci-joint le devis.',
        'Voir la pièce jointe.',
        'Les pièces jointes sont en haut',
        'Piece jointe : contrat',
        'Ci-jointe la facture',
        'Je vous joins le rapport',
        'Je joins mon CV',
        'Nous joignons les plans',
        'J’ai joint le fichier',
        'Le détail est en annexe.',
        'Document annexé',
        'Voir PJ',
        'Voir la P.J.',
        'Ci-attaché le fichier',
      ]) {
        expect(mentionsAttachment(s), isTrue, reason: s);
      }
    });

    test('en anglais', () {
      for (final s in [
        'Please find attached the report.',
        'See the attachment',
        'Attachments below',
        'I have enclosed my CV',
      ]) {
        expect(mentionsAttachment(s), isTrue, reason: s);
      }
    });
  });

  group('un texte qui n\'en annonce pas', () {
    test('mots qui contiennent seulement la racine', () {
      for (final s in [
        'Il rejoint l’équipe lundi.',
        'Mon conjoint arrive demain.',
        'Un joint de culasse',
        'Le pjsua est un outil',
        'Bonjour, merci pour votre retour.',
        '',
      ]) {
        expect(mentionsAttachment(s), isFalse, reason: s);
      }
    });
  });
}
