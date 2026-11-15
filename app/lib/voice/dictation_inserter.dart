//! Écrit le texte dicté dans un champ : partiel remplacé en direct, définitif validé, saisie de l'utilisateur respectée.

import 'package:flutter/widgets.dart';

/// Insère au curseur le texte reconnu, sans jamais écraser ce que l'utilisateur tape pendant ce temps.
class DictationInserter {
  DictationInserter(this.target);

  final TextEditingController target;
  int? _start;
  String _written = '';
  String _expectedText = '';
  int _expectedCursor = 0;
  String _lastRaw = '';
  String _skip = '';
  bool _released = false;
  bool _firstWrite = true;

  /// Remplace le texte provisoire (ou l'insère au curseur la première fois).
  void showPartial(String text) {
    if (_released) return;
    _followUser();
    _lastRaw = text;
    _write(_withoutCommitted(text), finalize: false);
  }

  /// Valide la phrase : le texte reste, suivi d'une espace, et la suite s'ancre au nouveau curseur.
  void commit(String text) {
    if (_released) return;
    _followUser();
    _write(_withoutCommitted(text), finalize: true);
  }

  /// Laisse le provisoire tel quel et ne touche plus jamais au champ.
  void release() => _released = true;

  /// Retire de la phrase cumulative ce que l'utilisateur a déjà vu validé dans le champ.
  String _withoutCommitted(String text) {
    if (_skip.isEmpty || !text.startsWith(_skip)) return text;
    return text.substring(_skip.length).trim();
  }

  /// Dit si l'utilisateur a modifié le texte ou déplacé le curseur depuis la dernière écriture.
  bool _touchedByUser() =>
      target.text != _expectedText ||
      !target.selection.isCollapsed ||
      target.selection.baseOffset != _expectedCursor;

  /// Si l'utilisateur a tapé ou déplacé le curseur, le partiel écrit devient du texte ordinaire.
  void _followUser() {
    if (_start != null && _touchedByUser()) _detach();
  }

  /// Abandonne l'ancrage courant : le partiel écrit devient du texte ordinaire.
  void _detach() {
    _skip = _lastRaw;
    _start = null;
    _written = '';
  }

  /// Pose l'ancrage ; seule la toute première écriture remplace une sélection, ensuite on écrit après elle.
  void _anchor() {
    final text = target.text;
    final sel = target.selection;
    final end = sel.isValid ? sel.end : text.length;
    final start = sel.isValid && _firstWrite ? sel.start : end;
    _firstWrite = false;
    _start = start;
    _written = text.substring(start, end);
  }

  /// Écrit [body] à la place du texte précédemment écrit par l'inserter.
  void _write(String body, {required bool finalize}) {
    if (_start == null && body.isEmpty) {
      if (finalize) _reset();
      return;
    }
    if (_start == null) _anchor();
    final text = target.text;
    final start = _start!;
    final before = text.substring(0, start);
    final after = text.substring(start + _written.length);
    final lead = body.isNotEmpty && _needsBlank(before) ? ' ' : '';
    final tail = _tail(body, after, finalize);
    final written = body.isEmpty ? '' : '$lead$body$tail';
    final cursor =
        start + lead.length + body.length + (finalize ? tail.length : 0);
    final next = '$before$written$after';
    target.value = TextEditingValue(
      text: next,
      selection: TextSelection.collapsed(offset: cursor),
    );
    _written = written;
    _expectedText = next;
    _expectedCursor = cursor;
    if (finalize) _reset();
  }

  /// Vrai si le texte qui précède se termine par un caractère qui n'est pas un blanc.
  bool _needsBlank(String before) =>
      before.isNotEmpty && !RegExp(r'\s').hasMatch(before[before.length - 1]);

  /// Espace à placer après le texte écrit, selon ce qui suit dans le champ.
  String _tail(String body, String after, bool finalize) {
    if (body.isEmpty) return '';
    if (after.isEmpty) return finalize ? ' ' : '';
    return RegExp(r'\s').hasMatch(after[0]) ? '' : ' ';
  }

  /// Fin de phrase : l'ancrage et la mémoire du cumul repartent de zéro.
  void _reset() {
    _start = null;
    _written = '';
    _lastRaw = '';
    _skip = '';
  }
}
