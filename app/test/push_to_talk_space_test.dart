import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mira/theme/app_theme.dart';
import 'package:mira/ui/push_to_talk_space.dart';
import 'package:mira/voice/dictation_types.dart';
import 'package:mira/voice/voice_services.dart';

import 'voice_fakes.dart';

const space = LogicalKeyboardKey.space;

class Rig {
  Rig(this.voice, this.field, this.focus, this.edits);
  final FakeVoice voice;
  final TextEditingController field;
  final FocusNode focus;
  final List<int> edits;
  DictationState get state => voice.services.dictation.state;
}

Future<Rig> mount(
  WidgetTester t, {
  String text = '',
  Set<DictationLanguage>? installed,
  bool withServices = true,
}) async {
  final voice = await FakeVoice.create(installed: installed);
  final field = TextEditingController(text: text)
    ..selection = TextSelection.collapsed(offset: text.length);
  final focus = FocusNode();
  final edits = <int>[];
  final page = MaterialApp(
    theme: buildTheme(false),
    home: Scaffold(
      body: PushToTalkSpace(
        target: field,
        onEdited: () => edits.add(1),
        child: TextField(controller: field, focusNode: focus),
      ),
    ),
  );
  await t.pumpWidget(
    withServices ? VoiceScope(services: voice.services, child: page) : page,
  );
  focus.requestFocus();
  await t.pump();
  return Rig(voice, field, focus, edits);
}

Future<void> settle(WidgetTester t) async {
  for (var i = 0; i < 8; i++) {
    await t.runAsync(
      () => Future<void>.delayed(const Duration(milliseconds: 10)),
    );
    await t.pump();
  }
}

Future<void> hold(WidgetTester t, [int ms = 450]) async {
  await t.sendKeyDownEvent(space);
  await t.pump(Duration(milliseconds: ms));
}

void main() {
  testWidgets('appui court : exactement un espace à la place du curseur', (
    t,
  ) async {
    final r = await mount(t, text: 'ab');
    await t.sendKeyDownEvent(space);
    await t.pump(const Duration(milliseconds: 80));
    expect(r.field.text, 'ab');
    await t.sendKeyUpEvent(space);
    await t.pump();
    expect(r.field.text, 'ab ');
    expect(r.field.selection, const TextSelection.collapsed(offset: 3));
    expect(r.state, DictationState.idle);
    expect(r.edits, hasLength(1));
    await t.runAsync(r.voice.dispose);
  });

  testWidgets('appui court sur une sélection : elle est remplacée', (t) async {
    final r = await mount(t, text: 'abcd');
    r.field.selection = const TextSelection(baseOffset: 1, extentOffset: 3);
    await t.sendKeyDownEvent(space);
    await t.sendKeyUpEvent(space);
    await t.pump();
    expect(r.field.text, 'a d');
    expect(r.field.selection, const TextSelection.collapsed(offset: 2));
    await t.runAsync(r.voice.dispose);
  });

  testWidgets('appui long : la dictée phrase par phrase démarre, sans espace', (
    t,
  ) async {
    final r = await mount(t, text: 'ab');
    await hold(t);
    expect(r.state, isNot(DictationState.idle));
    expect(r.voice.services.dictation.mode, DictationMode.pushToTalk);
    expect(r.voice.services.dictation.target, same(r.field));
    expect(r.field.text, 'ab');
    await t.sendKeyUpEvent(space);
    await settle(t);
    await t.runAsync(r.voice.dispose);
  });

  testWidgets('relâcher valide le texte dicté, sans espace en plus', (t) async {
    final r = await mount(t, text: 'ab');
    r.voice.recognizer.textOnStop = 'Bonjour.';
    await hold(t);
    await settle(t);
    await t.sendKeyUpEvent(space);
    await settle(t);
    expect(r.state, DictationState.idle);
    expect(r.field.text, 'ab Bonjour. ');
    await t.runAsync(r.voice.dispose);
  });

  testWidgets('rien reconnu : l’espace voulu est inséré au relâchement', (
    t,
  ) async {
    final r = await mount(t, text: 'ab');
    await hold(t);
    await settle(t);
    await t.sendKeyUpEvent(space);
    await settle(t);
    expect(r.field.text, 'ab ');
    await t.runAsync(r.voice.dispose);
  });

  testWidgets('les répétitions du clavier n’ajoutent aucun espace', (t) async {
    final r = await mount(t, text: 'ab');
    await hold(t);
    for (var i = 0; i < 5; i++) {
      await t.sendKeyRepeatEvent(space);
    }
    expect(r.field.text, 'ab');
    await t.sendKeyUpEvent(space);
    await settle(t);
    await t.runAsync(r.voice.dispose);
  });

  testWidgets(
    'une autre touche avant le seuil : l’espace passe d’abord, pas de dictée',
    (t) async {
      final r = await mount(t, text: 'x');
      await t.sendKeyDownEvent(space);
      await t.pump(const Duration(milliseconds: 60));
      await t.sendKeyDownEvent(LogicalKeyboardKey.keyA);
      expect(r.field.text, 'x ');
      await t.sendKeyUpEvent(LogicalKeyboardKey.keyA);
      await t.pump(const Duration(milliseconds: 600));
      expect(r.state, DictationState.idle);
      await t.sendKeyUpEvent(space);
      await t.pump();
      expect(r.field.text, 'x ');
      await t.runAsync(r.voice.dispose);
    },
  );

  testWidgets('le délai de maintien réglé est respecté', (t) async {
    final r = await mount(t);
    await r.voice.services.preferences.setHoldDelay(
      const Duration(milliseconds: 700),
    );
    await t.sendKeyDownEvent(space);
    await t.pump(const Duration(milliseconds: 500));
    expect(r.state, DictationState.idle);
    await t.pump(const Duration(milliseconds: 250));
    expect(r.state, isNot(DictationState.idle));
    await t.sendKeyUpEvent(space);
    await settle(t);
    await t.runAsync(r.voice.dispose);
  });

  testWidgets('réglage coupé : Espace n’est pas intercepté', (t) async {
    final r = await mount(t, text: 'ab');
    await r.voice.services.preferences.setHoldSpaceEnabled(false);
    await t.sendKeyDownEvent(space);
    await t.pump(const Duration(milliseconds: 600));
    await t.sendKeyUpEvent(space);
    await t.pump();
    expect(r.state, DictationState.idle);
    expect(r.field.text, 'ab');
    await t.runAsync(r.voice.dispose);
  });

  testWidgets(
    'modèle absent : Espace reste une touche ordinaire, aucune offre',
    (t) async {
      final r = await mount(t, text: 'ab', installed: {});
      await hold(t);
      expect(r.state, DictationState.idle);
      await t.sendKeyUpEvent(space);
      await t.pump();
      expect(r.field.text, 'ab');
      await t.runAsync(r.voice.dispose);
    },
  );

  testWidgets(
    'pendant une dictée continue, Espace reste une touche ordinaire',
    (t) async {
      final r = await mount(t, text: 'ab');
      await r.voice.services.dictation.start(r.field);
      await t.pump();
      await hold(t);
      expect(r.voice.services.dictation.mode, DictationMode.continuous);
      await t.sendKeyUpEvent(space);
      await t.pump();
      expect(r.state, DictationState.listening);
      await t.runAsync(r.voice.dispose);
    },
  );

  testWidgets('avec Ctrl, Espace n’est pas intercepté', (t) async {
    final r = await mount(t, text: 'ab');
    await t.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
    await t.sendKeyDownEvent(space);
    await t.pump(const Duration(milliseconds: 600));
    expect(r.state, DictationState.idle);
    await t.sendKeyUpEvent(space);
    await t.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
    await t.pump();
    expect(r.field.text, 'ab');
    await t.runAsync(r.voice.dispose);
  });

  testWidgets('pendant une composition de texte, Espace n’est pas intercepté', (
    t,
  ) async {
    final r = await mount(t, text: 'ab');
    r.field.value = const TextEditingValue(
      text: 'ab',
      selection: TextSelection.collapsed(offset: 2),
      composing: TextRange(start: 0, end: 2),
    );
    await hold(t);
    expect(r.state, DictationState.idle);
    await t.sendKeyUpEvent(space);
    await t.pump();
    await t.runAsync(r.voice.dispose);
  });

  testWidgets('sans services de dictée, Espace n’est jamais intercepté', (
    t,
  ) async {
    final r = await mount(t, text: 'ab', withServices: false);
    await hold(t);
    await t.sendKeyUpEvent(space);
    await t.pump();
    expect(r.field.text, 'ab');
    await t.runAsync(r.voice.dispose);
  });

  testWidgets('le champ perd le focus pendant la dictée : elle s’arrête', (
    t,
  ) async {
    final r = await mount(t, text: 'ab');
    await hold(t);
    await settle(t);
    r.focus.unfocus();
    await settle(t);
    expect(r.state, DictationState.idle);
    await t.runAsync(r.voice.dispose);
  });

  testWidgets(
    'le champ disparaît pendant la dictée : arrêt sans écriture ni plantage',
    (t) async {
      final r = await mount(t, text: 'ab');
      r.voice.recognizer.textOnStop = 'ne doit pas apparaître';
      await hold(t);
      await settle(t);
      await t.pumpWidget(const SizedBox());
      await settle(t);
      expect(r.state, DictationState.idle);
      expect(r.field.text, 'ab');
      await t.runAsync(r.voice.dispose);
    },
  );
}
