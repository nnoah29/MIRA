import 'package:flutter/material.dart';

import '../data/format.dart';
import '../engine/account_admin.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import '../theme/icons.dart';
import 'widgets.dart';

/// Ligne de la liste des comptes (mobile) : pastille, nom, adresse, état et flèche.
class AccountListRow extends StatelessWidget {
  final AccountDetail account;
  final String stateText;
  final bool bad;
  final VoidCallback onTap;
  const AccountListRow({
    super.key,
    required this.account,
    required this.stateText,
    required this.bad,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final a = account;
    final name = a.displayName.isEmpty ? a.address : a.displayName;
    return Hov(
      onTap: onTap,
      builder: (_, h) => Container(
        padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 14),
        decoration: BoxDecoration(
          color: h ? p.surface2 : null,
          border: Border(bottom: BorderSide(color: p.lineSoft)),
        ),
        child: Row(
          children: [
            _badge(a, name),
            const SizedBox(width: 14),
            Expanded(child: _texts(p, a, name)),
            RotatedBox(quarterTurns: 3, child: Ico('chev', color: p.muted)),
          ],
        ),
      ),
    );
  }

  /// Carré coloré portant les initiales du compte.
  Widget _badge(AccountDetail a, String name) => Container(
    width: 44,
    height: 44,
    alignment: Alignment.center,
    color: Color(colorOf(a.address)),
    child: Text(
      initials(name),
      style: Fonts.sans(
        size: 15,
        w: FontWeight.w700,
        color: const Color(0xFF1b1b1b),
      ),
    ),
  );

  /// Nom, adresse, état de connexion et actions en attente.
  Widget _texts(Pal p, AccountDetail a, String name) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Text(
        name,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
        style: Fonts.sans(size: 15, w: FontWeight.w500, color: p.text),
      ),
      Text(
        a.address,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
        style: Fonts.sans(size: 13, color: p.dim),
      ),
      const SizedBox(height: 4),
      Mono(
        [
          stateText,
          if (a.pendingActions > 0)
            t('{0} en attente', ['${a.pendingActions}']),
        ].join(' · '),
        size: 10,
        ls: .12,
        color: bad ? p.accent : p.muted,
        height: null,
        maxLines: 1,
      ),
    ],
  );
}

/// Ligne d'ajout de compte, en bas de la liste des comptes (mobile).
class AddAccountRow extends StatelessWidget {
  final VoidCallback onTap;
  const AddAccountRow({super.key, required this.onTap});

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Hov(
      onTap: onTap,
      builder: (_, h) => Container(
        padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 14),
        decoration: BoxDecoration(color: h ? p.surface2 : null),
        child: Row(
          children: [
            Container(
              width: 44,
              height: 44,
              alignment: Alignment.center,
              decoration: BoxDecoration(border: Border.all(color: p.accent)),
              child: Ico('plus', size: 18, color: p.accent),
            ),
            const SizedBox(width: 14),
            Text(
              t('Ajouter un compte'),
              style: Fonts.sans(size: 15, w: FontWeight.w500, color: p.accent),
            ),
          ],
        ),
      ),
    );
  }
}
