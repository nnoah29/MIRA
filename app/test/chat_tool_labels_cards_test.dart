import 'package:flutter_test/flutter_test.dart';
import 'package:mira/state/chat_message.dart';
import 'package:mira/ui/chat_tool_labels.dart';

void main() {
  test('the_show_mails_tool_has_a_readable_label', () {
    final tool = ToolActivity(id: 'c1', name: 'show_mails')
      ..state = ToolState.ok
      ..results = 3;
    expect(toolAction(tool), 'Cartes de messages');
    expect(toolLabel(tool), 'Cartes de messages · 3 résultats');
  });
}
