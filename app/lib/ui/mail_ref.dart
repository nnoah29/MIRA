//! Ligne cliquable d'un message cité par l'assistant : expéditeur, sujet, date ; un clic ouvre le mail.

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../data/format.dart';
import '../data/models.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import '../theme/icons.dart';
import 'widgets.dart';

/// Ligne d'un message cité ; [id] sert à retrouver la ligne dans les tests.
class MailRef extends StatelessWidget {
  final Mail m;
  const MailRef(this.m, {super.key});
  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final s = context.read<MailStore>();
    return Padding(
      padding: const EdgeInsets.only(bottom: 6),
      child: Hov(
        onTap: () => s.openRef(m.id),
        builder: (_, h) => Container(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          decoration: BoxDecoration(
            color: p.bg,
            border: Border.all(color: h ? p.accent : p.line),
          ),
          child: Row(
            children: [
              Avatar(m.from.name, size: 30, font: 10),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      m.subject,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: Fonts.sans(
                        size: 14,
                        w: FontWeight.w500,
                        color: p.text,
                        height: 1.3,
                      ),
                    ),
                    Text(
                      '${m.from.name} · ${rowTime(m.date)}',
                      style: Fonts.sans(size: 12, color: p.dim),
                    ),
                  ],
                ),
              ),
              Transform.rotate(
                angle: 3.14159,
                child: Ico('arrowl', size: 14, color: p.dim),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
