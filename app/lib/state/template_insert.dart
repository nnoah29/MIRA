import 'package:flutter/widgets.dart';

import 'rich_controller.dart';

/// Un « / » vient-il d'être tapé juste avant le curseur, en début de ligne ou après un blanc ?
bool slashOpensPicker(String text, int cursor) {
  if (cursor < 1 || cursor > text.length || text[cursor - 1] != '/') {
    return false;
  }
  return cursor == 1 || ' \n\t'.contains(text[cursor - 2]);
}

/// Le texte vient-il de gagner exactement un « / » tapé juste avant le curseur (et non un collage ou un déplacement) ?
bool slashJustTyped(String before, String after, int cursor) =>
    after.length == before.length + 1 && slashOpensPicker(after, cursor);

/// Zone remplacée par une insertion : le « / » déclencheur s'il y est encore, sinon la sélection (ou la fin).
(int, int) _replaced(RichController body, int? slashAt) {
  final text = body.text;
  final at = slashAt;
  if (at != null && at >= 0 && at < text.length && text[at] == '/') {
    return (at, at + 1);
  }
  final sel = body.selection;
  return sel.isValid ? (sel.start, sel.end) : (text.length, text.length);
}

/// Insère [text] à la place du « / » tapé en [slashAt], sinon de la sélection (ou à la fin).
void insertAtCursor(RichController body, String text, {int? slashAt}) {
  final (start, end) = _replaced(body, slashAt);
  body.value = TextEditingValue(
    text: body.text.replaceRange(start, end, text),
    selection: TextSelection.collapsed(offset: start + text.length),
  );
}
