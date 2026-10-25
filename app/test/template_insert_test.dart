import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/rich_controller.dart';
import 'package:mira/state/template_insert.dart';

void main() {
  group('slashOpensPicker', () {
    test('vrai en début de texte et en début de ligne', () {
      expect(slashOpensPicker('/', 1), isTrue);
      expect(slashOpensPicker('Bonjour\n/', 9), isTrue);
    });

    test('vrai après un espace', () {
      expect(slashOpensPicker('bonjour /', 9), isTrue);
    });

    test('faux dans une adresse, un chemin ou sans barre', () {
      expect(slashOpensPicker('http://', 7), isFalse);
      expect(slashOpensPicker('a/b', 2), isFalse);
      expect(slashOpensPicker('bonjour', 7), isFalse);
      expect(slashOpensPicker('', 0), isFalse);
    });

    test('faux si le curseur n’est pas juste après la barre', () {
      expect(slashOpensPicker('/ texte', 5), isFalse);
      expect(slashOpensPicker('/', -1), isFalse);
    });
  });

  group('slashJustTyped', () {
    test('vrai quand un seul « / » vient d’être ajouté au bon endroit', () {
      expect(slashJustTyped('bonjour ', 'bonjour /', 9), isTrue);
      expect(slashJustTyped('', '/', 1), isTrue);
    });

    test(
      'faux si le texte n’a pas changé (curseur déplacé après un « / »)',
      () {
        expect(slashJustTyped('bonjour /', 'bonjour /', 9), isFalse);
      },
    );

    test('faux pour un collage ou un brouillon rechargé', () {
      expect(slashJustTyped('a ', 'a b /', 5), isFalse);
      expect(slashJustTyped('', 'Texte /', 7), isFalse);
    });

    test('faux si le caractère ajouté n’est pas une barre oblique', () {
      expect(slashJustTyped('bonjour ', 'bonjour x', 9), isFalse);
    });

    test('faux pour une barre dans une adresse', () {
      expect(slashJustTyped('http:/', 'http://', 7), isFalse);
    });
  });

  group('insertAtCursor', () {
    RichController body(String text, int at) =>
        RichController()
          ..value = TextEditingValue(
            text: text,
            selection: TextSelection.collapsed(offset: at),
          );

    test('insère à la position du curseur et le replace après le texte', () {
      final b = body('Bonjour !', 7);
      insertAtCursor(b, ' Léa');
      expect(b.text, 'Bonjour Léa !');
      expect(b.selection.baseOffset, 11);
    });

    test('remplace la sélection', () {
      final b = body('Bonjour monde', 8)
        ..selection = const TextSelection(baseOffset: 8, extentOffset: 13);
      insertAtCursor(b, 'Léa');
      expect(b.text, 'Bonjour Léa');
    });

    test('retire la barre oblique déclencheuse', () {
      final b = body('Salut /', 7);
      insertAtCursor(b, 'Texte', slashAt: 6);
      expect(b.text, 'Salut Texte');
    });

    test('garde une barre qui n’est pas juste avant le curseur', () {
      final b = body('a/b', 3);
      insertAtCursor(b, 'X', slashAt: 0);
      expect(b.text, 'a/bX');
    });

    test('remplace la barre déclencheuse même si le curseur a avancé', () {
      final b = body('Salut /m', 8);
      insertAtCursor(b, 'Texte', slashAt: 6);
      expect(b.text, 'Salut Textem');
      expect(b.selection.baseOffset, 11);
    });

    test('sans sélection valide, ajoute à la fin', () {
      final b = RichController()..text = 'Fin';
      insertAtCursor(b, '!');
      expect(b.text, 'Fin!');
    });
  });
}
