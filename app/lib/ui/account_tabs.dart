import 'package:flutter/material.dart';

import '../data/format.dart';
import '../engine/account_admin.dart';
import '../i18n/i18n.dart';
import '../theme/icons.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

const double _tabHeight = 84;

/// Cadre commun des onglets : hauteur fixe, carré à gauche aussi haut que le texte.
class _TabFrame extends StatelessWidget {
  final Widget square, child;
  final Color? fill;
  final Color underline;
  final double underlineWidth;
  const _TabFrame({
    required this.square,
    required this.child,
    required this.fill,
    required this.underline,
    required this.underlineWidth,
  });

  @override
  Widget build(BuildContext context) => Container(
    width: 240,
    height: _tabHeight,
    padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 12),
    decoration: BoxDecoration(
      color: fill,
      border: Border(
        bottom: BorderSide(color: underline, width: underlineWidth),
      ),
    ),
    child: Row(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        AspectRatio(aspectRatio: 1, child: square),
        const SizedBox(width: 12),
        Expanded(child: child),
      ],
    ),
  );
}

/// Onglet d'un compte : pastille, nom, adresse, état et actions en attente.
class AccountTab extends StatelessWidget {
  final AccountDetail account;
  final String stateText;
  final bool bad, selected;
  final VoidCallback onTap;
  const AccountTab({
    super.key,
    required this.account,
    required this.stateText,
    required this.bad,
    required this.selected,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final a = account;
    final name = a.displayName.isEmpty ? a.address : a.displayName;
    return Hov(
      onTap: onTap,
      builder: (_, h) => _TabFrame(
        fill: selected ? p.accentSoft : (h ? p.surface2 : null),
        underline: selected ? p.accent : p.lineSoft,
        underlineWidth: selected ? 2 : 1,
        square: Container(
          alignment: Alignment.center,
          color: Color(colorOf(a.address)),
          child: Text(
            initials(name),
            style: Fonts.sans(
              size: 14,
              w: FontWeight.w700,
              color: const Color(0xFF1b1b1b),
            ),
          ),
        ),
        child: _texts(p, name),
      ),
    );
  }

  Widget _texts(Pal p, String name) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    mainAxisAlignment: MainAxisAlignment.spaceBetween,
    children: [
      Text(
        name,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
        style: Fonts.sans(
          size: 14,
          w: FontWeight.w500,
          color: selected ? p.accent : p.text,
        ),
      ),
      Text(
        account.address,
        maxLines: 1,
        overflow: TextOverflow.ellipsis,
        style: Fonts.sans(size: 12, color: p.dim),
      ),
      Row(
        children: [
          Flexible(
            child: Mono(
              stateText,
              size: 10,
              ls: .12,
              color: bad ? p.accent : p.muted,
              height: null,
              maxLines: 1,
            ),
          ),
          if (account.pendingActions > 0) ...[
            const SizedBox(width: 8),
            Flexible(
              child: Mono(
                t('{0} en attente', ['${account.pendingActions}']),
                size: 10,
                color: p.muted,
                height: null,
                maxLines: 1,
              ),
            ),
          ],
        ],
      ),
    ],
  );
}

/// Onglet d'ajout de compte, de même format que les onglets des comptes.
class AddAccountTab extends StatelessWidget {
  final bool selected;
  final VoidCallback onTap;
  const AddAccountTab({super.key, required this.selected, required this.onTap});

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Hov(
      onTap: onTap,
      builder: (_, h) => _TabFrame(
        fill: selected ? p.accentSoft : (h ? p.surface2 : null),
        underline: selected ? p.accent : p.lineSoft,
        underlineWidth: selected ? 2 : 1,
        square: Container(
          alignment: Alignment.center,
          decoration: BoxDecoration(border: Border.all(color: p.accent)),
          child: Ico('plus', size: 16, color: p.accent),
        ),
        child: Align(
          alignment: Alignment.centerLeft,
          child: Text(
            t('Ajouter un compte'),
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: Fonts.sans(size: 14, w: FontWeight.w500, color: p.accent),
          ),
        ),
      ),
    );
  }
}
