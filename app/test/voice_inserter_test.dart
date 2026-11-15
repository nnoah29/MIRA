import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/rich_controller.dart';
import 'package:mira/voice/dictation_inserter.dart';

TextEditingController field(String text, [int? cursor]) {
  final c = TextEditingController(text: text);
  c.selection = TextSelection.collapsed(offset: cursor ?? text.length);
  return c;
}

void main() {
  test('le partiel est remplacé à chaque mise à jour puis validé', () {
    final c = field('Bonjour ');
    final ins = DictationInserter(c);
    ins.showPartial('je');
    expect(c.text, 'Bonjour je');
    ins.showPartial('je vous');
    expect(c.text, 'Bonjour je vous');
    ins.commit('je vous confirme.');
    expect(c.text, 'Bonjour je vous confirme. ');
    expect(c.selection.baseOffset, c.text.length);
  });

  test(
    'une espace est ajoutée si le caractère précédent n’est pas un blanc',
    () {
      final c = field('Bonjour');
      DictationInserter(c).showPartial('je');
      expect(c.text, 'Bonjour je');
    },
  );

  test('aucune espace en tête sur un champ vide', () {
    final c = field('');
    DictationInserter(c).showPartial('je');
    expect(c.text, 'je');
  });

  test('insertion au milieu du texte', () {
    final c = field('Salut  fin', 6);
    DictationInserter(c).showPartial('toi');
    expect(c.text, 'Salut toi fin');
    expect(c.selection.baseOffset, 'Salut toi'.length);
  });

  test('une sélection non vide est remplacée', () {
    final c = field('un mot ici');
    c.selection = const TextSelection(baseOffset: 3, extentOffset: 6);
    DictationInserter(c).showPartial('nom');
    expect(c.text, 'un nom ici');
  });

  test('la frappe pendant un partiel est conservée', () {
    final c = field('A ');
    final ins = DictationInserter(c);
    ins.showPartial('bc');
    expect(c.text, 'A bc');
    c.value = const TextEditingValue(
      text: 'A bcX',
      selection: TextSelection.collapsed(offset: 5),
    );
    ins.showPartial('de');
    expect(c.text, 'A bcX de');
  });

  test('après une frappe, le partiel cumulatif n’est pas réécrit', () {
    final c = field('A ');
    final ins = DictationInserter(c);
    ins.showPartial('bc');
    c.value = const TextEditingValue(
      text: 'A bcX',
      selection: TextSelection.collapsed(offset: 5),
    );
    ins.showPartial('bc de');
    expect(c.text, 'A bcX de');
    ins.commit('bc de.');
    expect(c.text, 'A bcX de. ');
  });

  test('déplacer le curseur pendant un partiel réancre la suite', () {
    final c = field('Un deux ');
    final ins = DictationInserter(c);
    ins.showPartial('trois');
    c.selection = const TextSelection.collapsed(offset: 0);
    ins.showPartial('quatre');
    expect(c.text, startsWith('quatre'));
    expect(c.text, contains('Un deux trois'));
  });

  test('release valide le partiel et ne touche plus au champ', () {
    final c = field('A ');
    final ins = DictationInserter(c);
    ins.showPartial('bc');
    ins.release();
    ins.showPartial('zzz');
    ins.commit('zzz');
    expect(c.text, 'A bc');
  });

  test('commit vide retire le partiel sans laisser d’espace', () {
    final c = field('A');
    final ins = DictationInserter(c);
    ins.showPartial('bc');
    ins.commit('');
    expect(c.text, 'A');
  });

  test('dans un champ de texte riche, les styles restent alignés', () {
    final c = RichController();
    c.setContent('Bonjour', List<int>.filled(7, 0));
    c.selection = const TextSelection.collapsed(offset: 7);
    final ins = DictationInserter(c);
    ins.showPartial('je');
    ins.commit('je vous écris.');
    expect(c.text, 'Bonjour je vous écris. ');
    expect(c.flags.length, c.text.length);
  });

  test('une sélection faite pendant la dictée n’est jamais remplacée', () {
    final c = field('Texte important. ');
    final ins = DictationInserter(c);
    ins.showPartial('je');
    c.selection = const TextSelection(baseOffset: 0, extentOffset: 15);
    ins.showPartial('vous');
    expect(c.text, contains('Texte important'));
    expect(c.text, contains('vous'));
  });

  test('après une frappe puis une fin de phrase sans mot nouveau, la phrase suivante est entière', () {
    final c = field('A ');
    final ins = DictationInserter(c);
    ins.showPartial('Merci');
    c.value = const TextEditingValue(
      text: 'A MerciX',
      selection: TextSelection.collapsed(offset: 8),
    );
    ins.commit('Merci');
    ins.showPartial('Merci pour votre retour');
    expect(c.text, contains('Merci pour votre retour'));
  });
}
