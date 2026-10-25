import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../engine/sync_alert.dart';
import '../engine/sync_state.dart';
import '../i18n/i18n.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

/// Bandeau sous la barre du haut : hors ligne, échec de synchronisation, ou avancement de la synchro.
class SyncBanner extends StatelessWidget {
  const SyncBanner({super.key});

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    final p = context.pal;
    final issues = s.syncIssues;
    final urgent = mostUrgent(issues);
    if (urgent == null) return _progress(issues, p);
    final (account, state) = urgent;
    return Container(
      key: const ValueKey('sync-banner'),
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 6),
      color: p.lineSoft,
      child: Row(
        children: [
          Expanded(
            child: Text(
              syncAlertMessage(account.email, state),
              style: Fonts.sans(size: 13, color: p.text),
            ),
          ),
          Btn(label: t('Réessayer'), onTap: s.retrySync),
        ],
      ),
    );
  }

  Widget _progress(List<(dynamic, SyncState)> issues, Pal p) {
    final pct = issues.map((e) => e.$2.percent).whereType<int>().firstOrNull;
    if (pct == null) return const SizedBox.shrink();
    return LinearProgressIndicator(
      key: const ValueKey('sync-progress'),
      value: pct / 100,
      minHeight: 2,
      color: p.accent,
      backgroundColor: p.lineSoft,
    );
  }
}
