import 'dart:async';

import 'package:flutter/material.dart';

import '../data/format.dart';
import '../engine/account_admin.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

/// Ligne d'état sous un compte : dernière synchronisation réussie et dernière erreur.
class SyncStatusLine extends StatefulWidget {
  final AccountDetail account;
  final AccountAdmin admin;
  const SyncStatusLine({super.key, required this.account, required this.admin});

  @override
  State<SyncStatusLine> createState() => _SyncStatusLineState();
}

class _SyncStatusLineState extends State<SyncStatusLine> {
  SyncStatus? status;

  @override
  void initState() {
    super.initState();
    unawaited(_load());
  }

  Future<void> _load() async {
    try {
      final all = await widget.admin.syncStatus();
      if (mounted) setState(() => status = all[widget.account.id]);
    } catch (_) {}
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final st = status;
    if (st == null) return const SizedBox.shrink();
    final parts = [
      if (st.syncing)
        t('Synchronisation en cours')
      else if (st.lastSyncedAt != null)
        t('Dernière synchronisation : {0}', [rowTime(st.lastSyncedAt!)])
      else
        t('Jamais synchronisé'),
      if (st.lastError != null) t('Erreur : {0}', [st.lastError!]),
    ];
    return Padding(
      padding: const EdgeInsets.only(top: 10),
      child: Mono(
        parts.join(' · '),
        key: ValueKey('sync-status-${widget.account.id}'),
        size: 10.5,
        height: null,
        color: st.lastError != null ? p.accent : p.dim,
      ),
    );
  }
}
