//! Pastilles d'activité de l'IA sous sa réponse : ce qu'elle a consulté, avec « Autoriser… » sous un refus.

import 'package:flutter/material.dart';

import '../i18n/i18n.dart';
import '../state/chat_message.dart';
import '../theme/app_theme.dart';
import '../theme/icons.dart';
import 'chat_tool_labels.dart';
import 'widgets.dart';

/// Une pastille par appel d'outil, dans l'ordre ; le bouton d'autorisation s'affiche sous la pastille refusée.
class ChatToolChips extends StatelessWidget {
  final List<ToolActivity> tools;
  final bool canAuthorize;
  final void Function(ToolActivity tool) onAuthorize;
  const ChatToolChips({
    super.key,
    required this.tools,
    required this.canAuthorize,
    required this.onAuthorize,
  });

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        for (final tool in tools)
          Padding(
            padding: const EdgeInsets.only(top: 8),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                _Chip(tool),
                if (tool.state == ToolState.denied &&
                    tool.deniedKind != null &&
                    canAuthorize)
                  Padding(
                    padding: const EdgeInsets.only(top: 8),
                    child: Btn(
                      key: ValueKey('tool-authorize-${tool.id}'),
                      label: t('Autoriser…'),
                      onTap: () => onAuthorize(tool),
                    ),
                  ),
              ],
            ),
          ),
      ],
    );
  }
}

class _Chip extends StatelessWidget {
  final ToolActivity tool;
  const _Chip(this.tool);

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final failed =
        tool.state == ToolState.denied || tool.state == ToolState.error;
    return Container(
      key: ValueKey('tool-chip-${tool.id}'),
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: BoxDecoration(
        color: p.surface2,
        border: Border.all(color: failed ? p.accent : p.lineSoft),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.center,
        children: [
          Ico('search', size: 13, color: p.dim),
          const SizedBox(width: 8),
          Flexible(
            child: Text(
              toolLabel(tool),
              style: Fonts.sans(size: 13, color: p.dim),
            ),
          ),
        ],
      ),
    );
  }
}
