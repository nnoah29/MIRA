//! Menu du bouton de jonction de l'assistant : liste d'options extensible, dont « Joindre un mail ».

import 'package:flutter/material.dart';

import '../theme/app_theme.dart';
import '../theme/icons.dart';
import 'widgets.dart';

/// Une option du menu de jonction.
class ChatAttachOption {
  final String id, icon, label;
  final VoidCallback onTap;
  const ChatAttachOption({
    required this.id,
    required this.icon,
    required this.label,
    required this.onTap,
  });
}

/// Panneau d'options ; d'autres entrées (fichier…) s'ajoutent en allongeant la liste.
class ChatAttachMenu extends StatelessWidget {
  final List<ChatAttachOption> options;
  const ChatAttachMenu({super.key, required this.options});

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Column(
      key: const ValueKey('chat-attach-menu'),
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [for (final o in options) _row(p, o)],
    );
  }

  Widget _row(Pal p, ChatAttachOption o) => Hov(
    onTap: o.onTap,
    builder: (_, h) => Container(
      key: ValueKey('chat-attach-option-${o.id}'),
      color: h ? p.surface : null,
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 10),
      child: Row(
        children: [
          Ico(o.icon, size: 15, color: h ? p.accent : p.muted),
          const SizedBox(width: 10),
          Text(o.label, style: Fonts.sans(size: 14, color: p.text)),
        ],
      ),
    ),
  );
}
