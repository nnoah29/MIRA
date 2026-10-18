import 'dart:async';

import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../engine/account_admin.dart';
import '../i18n/i18n.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import '../theme/icons.dart';
import 'widgets.dart';

/// Choix des dossiers que le moteur synchronise pour un compte ; la réception reste toujours synchronisée.
class SyncedMailboxes extends StatefulWidget {
  final AccountDetail account;
  final AccountAdmin admin;
  const SyncedMailboxes({
    super.key,
    required this.account,
    required this.admin,
  });

  @override
  State<SyncedMailboxes> createState() => _SyncedMailboxesState();
}

class _SyncedMailboxesState extends State<SyncedMailboxes> {
  List<BoxChoice>? boxes;
  bool busy = false;

  @override
  void initState() {
    super.initState();
    unawaited(_load());
  }

  Future<void> _load() async {
    try {
      final got = await widget.admin.mailboxes(widget.account.id);
      if (mounted) setState(() => boxes = got);
    } catch (_) {
      if (mounted) setState(() => boxes = const []);
    }
  }

  Future<void> _toggle(BoxChoice b) async {
    final all = boxes!;
    final next = [
      for (final x in all)
        if (x.id == b.id) BoxChoice(x.id, x.name, x.role, !x.synced) else x,
    ];
    final s = context.read<MailStore>();
    setState(() {
      boxes = next;
      busy = true;
    });
    try {
      await widget.admin.setSyncedMailboxes(widget.account.id, [
        for (final x in next)
          if (x.synced) x.id,
      ]);
      s.toast(t('Dossiers synchronisés modifiés'));
    } catch (_) {
      if (mounted) setState(() => boxes = all);
      s.toast(t('Modification impossible'), err: true);
    }
    if (mounted) setState(() => busy = false);
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final list = boxes;
    if (list == null) {
      return Mono(t('Chargement…'), color: p.dim, height: null);
    }
    return Wrap(
      spacing: 8,
      runSpacing: 8,
      children: [
        for (final b in list)
          Hov(
            key: ValueKey('synced-${b.id}'),
            onTap: busy || b.role == 'inbox' ? null : () => _toggle(b),
            builder: (_, h) => Container(
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
              decoration: BoxDecoration(
                color: b.synced ? p.accentSoft : null,
                border: Border.all(color: b.synced ? p.accent : p.line),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  if (b.synced) ...[
                    Ico('check', size: 12, color: p.accent),
                    const SizedBox(width: 6),
                  ],
                  Text(
                    b.name,
                    style: Fonts.sans(
                      size: 13,
                      color: b.synced ? p.text : (h ? p.text : p.dim),
                    ),
                  ),
                ],
              ),
            ),
          ),
      ],
    );
  }
}
