//! Application d'une proposition de l'IA au brouillon ouvert, et instantané pour l'annuler exactement.

import 'package:flutter/widgets.dart';

import '../data/models.dart';
import '../engine/ai_models.dart';
import '../i18n/i18n.dart';
import 'draft_ai_request.dart';
import 'rich_controller.dart';
import 'store.dart';

/// État du brouillon avant l'IA : objet, destinataires, texte, mise en forme et curseur.
class DraftSnapshot {
  final String subject;
  final String body;
  final List<int> flags;
  final List<Person> to;
  final List<Person> cc;
  final TextSelection selection;
  const DraftSnapshot._(
    this.subject,
    this.body,
    this.flags,
    this.to,
    this.cc,
    this.selection,
  );

  /// Copie l'état courant du brouillon.
  factory DraftSnapshot.of(ComposeState c) => DraftSnapshot._(
    c.subject.text,
    c.body.text,
    List<int>.from(c.body.flags),
    [...c.to],
    [...c.cc],
    c.body.selection,
  );

  /// Remet le brouillon dans cet état.
  void restore(ComposeState c) {
    c.subject.text = subject;
    c.to = [...to];
    c.cc = [...cc];
    c.body.setContent(body, flags);
    c.body.selection = selection;
  }
}

List<Person> _people(List<AiAddressEntry> list) => [
  for (final e in list) Person(e.name ?? e.address, e.address),
];

void _setBody(ComposeState c, String body) {
  final text = c.body.text;
  final flags = c.body.flags;
  final at = authoredEnd(c);
  final head = at >= text.length ? body : '$body\n\n';
  final tail = flags.length > at ? flags.sublist(at) : const <int>[];
  c.body.setContent('$head${text.substring(at)}', [
    ...RichController.gapFlags(head),
    ...tail,
  ]);
  c.body.selection = TextSelection.collapsed(offset: body.length);
}

String? _replaceSelection(ComposeState c, String text, DraftSelection? from) {
  if (from == null) {
    return t('Aucun passage n’est sélectionné.');
  }
  final body = c.body.text;
  final intact =
      from.end <= body.length &&
      body.substring(from.start, from.end) == from.text;
  final start = intact ? from.start : body.indexOf(from.text);
  if (start < 0) {
    return t('Le passage sélectionné a changé : relance l’action.');
  }
  if (!intact && start != body.lastIndexOf(from.text)) {
    return t(
      'Le passage sélectionné apparaît plusieurs fois : relance l’action.',
    );
  }
  c.body.value = TextEditingValue(
    text: body.replaceRange(start, start + from.text.length, text),
    selection: TextSelection(
      baseOffset: start,
      extentOffset: start + text.length,
    ),
  );
  return null;
}

/// Applique la proposition au brouillon ; rend le texte d'une erreur, ou `null` si c'est fait.
String? applyProposal(ComposeState c, AiDraftProposal p, DraftSelection? from) {
  final selected = p.selection;
  if (selected != null) {
    return _replaceSelection(c, selected, from);
  }
  final (subject, to, cc, body) = (p.subject, p.to, p.cc, p.body);
  if (subject != null) {
    c.subject.text = subject;
  }
  if (to != null) {
    c.to = _people(to);
  }
  if (cc != null) {
    c.cc = _people(cc);
  }
  if (body != null) {
    _setBody(c, body);
  }
  return null;
}

String _who(AiAddressEntry e) =>
    e.name == null ? e.address : '${e.name} <${e.address}>';

/// Ce que la proposition a changé, avec les vraies adresses : l'utilisateur voit à qui le message ira.
String changesOf(AiDraftProposal p) {
  final parts = [
    if (p.subject != null) '${t('Objet')} : ${p.subject}',
    if (p.to != null) '${t('À')} : ${p.to!.map(_who).join(', ')}',
    if (p.cc != null) '${t('Cc')} : ${p.cc!.map(_who).join(', ')}',
    if (p.body != null) t('Corps'),
    if (p.selection != null) t('Passage remplacé'),
  ];
  return parts.join(' · ');
}
