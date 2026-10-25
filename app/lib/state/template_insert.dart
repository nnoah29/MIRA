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

/// Insère [text] à la place de la sélection (ou à la fin), en retirant le « / » déclencheur si demandé.
void insertAtCursor(
  RichController body,
  String text, {
  bool removeSlash = false,
}) {
  final sel = body.selection;
  var start = sel.isValid ? sel.start : body.text.length;
  final end = sel.isValid ? sel.end : body.text.length;
  if (removeSlash && start > 0 && body.text[start - 1] == '/') start -= 1;
  body.value = TextEditingValue(
    text: body.text.replaceRange(start, end, text),
    selection: TextSelection.collapsed(offset: start + text.length),
  );
}
