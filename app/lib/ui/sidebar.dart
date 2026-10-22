import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../data/models.dart';
import '../state/local_labels.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import '../theme/icons.dart';
import 'mira_logo.dart';
import 'overlays.dart';
import 'widgets.dart';
import '../i18n/i18n.dart';

class Sidebar extends StatelessWidget {
  final double width;
  const Sidebar({super.key, this.width = 248});

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    final p = context.pal;
    final un = s.navCount('inbox');
    return Container(
      width: width,
      decoration: BoxDecoration(
        color: p.bgSide,
        border: Border(right: BorderSide(color: p.lineSoft)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(24, 22, 24, 0),
            child: Row(
              children: [
                MiraLogo(size: 48, color: p.accent),
                const SizedBox(width: 14),
                Text.rich(
                  TextSpan(
                    text: 'MI',
                    children: [
                      TextSpan(
                        text: 'RA',
                        style: TextStyle(color: p.accent),
                      ),
                    ],
                  ),
                  style: Fonts.display(size: 30, color: p.text, ls: .06),
                ),
              ],
            ),
          ),
          Padding(
            padding: const EdgeInsets.fromLTRB(24, 10, 24, 18),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                AnimatedCount(
                  un,
                  style: Fonts.display(size: 52, color: p.text),
                ),
                const SizedBox(height: 4),
                Mono(t('Non lus')),
                const SizedBox(height: 12),
                Container(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 12,
                    vertical: 7,
                  ),
                  decoration: BoxDecoration(
                    color: p.surface,
                    border: Border.all(color: p.line),
                  ),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Square(8, p.muted),
                      const SizedBox(width: 8),
                      Mono(t('{0} à répondre', [s.needsCount]), color: p.muted),
                    ],
                  ),
                ),
              ],
            ),
          ),
          Padding(
            padding: const EdgeInsets.fromLTRB(20, 0, 20, 4),
            child: Hov(
              onTap: () => s.openCompose(const ComposeInit()),
              press: true,
              builder: (_, h) {
                Widget b = Container(
                  padding: const EdgeInsets.all(13),
                  color: p.accent,
                  child: Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Ico('pen', color: p.onAccent),
                      const SizedBox(width: 10),
                      Mono(
                        t('Nouveau message'),
                        w: FontWeight.w700,
                        color: p.onAccent,
                        height: null,
                      ),
                    ],
                  ),
                );
                if (h) {
                  b = ColorFiltered(
                    colorFilter: const ColorFilter.matrix([
                      1.1,
                      0,
                      0,
                      0,
                      0,
                      0,
                      1.1,
                      0,
                      0,
                      0,
                      0,
                      0,
                      1.1,
                      0,
                      0,
                      0,
                      0,
                      0,
                      1,
                      0,
                    ]),
                    child: b,
                  );
                }
                return b;
              },
            ),
          ),
          Expanded(
            child: ListView(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
              children: [
                _NavTitle(t('Ma boîte')),
                for (final f in folderDefs)
                  if (f.id != 'outbox' ||
                      s.navCount('outbox') > 0 ||
                      (s.label == null && s.folder == 'outbox'))
                    _NavItem(
                      on: s.label == null && s.folder == f.id,
                      onTap: () => s.selectFolder(f.id),
                      leading: Ico(
                        f.icon,
                        color: s.label == null && s.folder == f.id
                            ? p.accent
                            : null,
                      ),
                      label: t(f.n),
                      count: s.navCount(f.id),
                    ),
                ..._labelGroup(s, t('Libellés locaux'), true),
                ..._labelGroup(s, t('Libellés'), false),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

/// Titre puis entrées des libellés locaux (`local`) ou de ceux des comptes ; rien s'il n'y en a pas.
List<Widget> _labelGroup(MailStore s, String title, bool local) {
  final items = [
    for (final e in s.sidebarLabels())
      if (isLocalLabel(e.key) == local) e,
  ];
  if (items.isEmpty) return const [];
  return [
    _NavTitle(title),
    for (final e in items)
      _NavItem(
        on: s.label == e.key,
        onTap: () => s.selectLabel(e.key),
        leading: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 4),
          child: local
              ? Ico(
                  'diamond',
                  size: 8,
                  color: Color(s.labelColor(e.key, e.value.c)),
                )
              : Square(8, Color(s.labelColor(e.key, e.value.c))),
        ),
        label: s.labelTitle(e.key),
        count: s.labelCount(e.key),
      ),
  ];
}

class _NavTitle extends StatelessWidget {
  final String t;
  const _NavTitle(this.t);
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.fromLTRB(12, 12, 12, 10),
    child: Mono(t, size: 10.5, ls: .18, height: null),
  );
}

class _NavItem extends StatelessWidget {
  final bool on;
  final VoidCallback onTap;
  final Widget leading;
  final String label;
  final int count;
  const _NavItem({
    required this.on,
    required this.onTap,
    required this.leading,
    required this.label,
    required this.count,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Hov(
      onTap: onTap,
      builder: (_, h) {
        final fg = on || h ? p.text : p.muted;
        return AnimatedContainer(
          duration: const Duration(milliseconds: 130),
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8.25),
          decoration: BoxDecoration(
            color: on ? p.surface2 : (h ? p.surface : Colors.transparent),
            border: Border(
              left: BorderSide(
                color: on ? p.accent : Colors.transparent,
                width: 2,
              ),
            ),
          ),
          child: DefaultTextStyle.merge(
            style: TextStyle(color: p.muted),
            child: Row(
              children: [
                leading,
                const SizedBox(width: 14),
                Expanded(
                  child: Text(label, style: Fonts.sans(size: 15, color: fg)),
                ),
                if (count > 0)
                  AnimatedCount(
                    count,
                    style: Fonts.mono(
                      size: 11,
                      w: FontWeight.w500,
                      color: on ? p.accent : p.dim,
                      ls: 0,
                      height: 1.5,
                    ),
                  ),
              ],
            ),
          ),
        );
      },
    );
  }
}

class Topbar extends StatelessWidget {
  const Topbar({super.key});

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    final p = context.pal;
    final w = MediaQuery.sizeOf(context).width;
    final mobile = w <= 760;
    final hasQ = s.q.isNotEmpty;
    final wide = w > 1350;
    return Container(
      height: 64,
      padding: EdgeInsets.symmetric(horizontal: mobile ? 12 : 32),
      decoration: BoxDecoration(
        border: Border(bottom: BorderSide(color: p.lineSoft)),
      ),
      child: Row(
        children: [
          if (mobile) ...[
            Transform.translate(
              offset: const Offset(-6, 0),
              child: IBtn(
                'list',
                size: 20,
                onTap: s.toggleNav,
                tip: t('Ouvrir le menu'),
                color: p.text,
              ),
            ),
            const SizedBox(width: 6),
          ],
          Expanded(
            child: Row(
              children: [
                Ico('search', color: p.muted),
                const SizedBox(width: 14),
                Expanded(
                  child: TextField(
                    controller: s.searchCtl,
                    focusNode: s.searchFocus,
                    onChanged: s.setQuery,
                    cursorColor: p.accent,
                    cursorWidth: 1,
                    style: Fonts.sans(size: 15, color: p.text, height: 1.5),
                    decoration: InputDecoration(
                      isDense: true,
                      border: InputBorder.none,
                      contentPadding: EdgeInsets.zero,
                      hintText: t('Chercher dans mes messages…'),
                      hintStyle: Fonts.sans(size: 15, color: p.muted),
                    ),
                  ),
                ),
                IBtn(
                  'filter',
                  key: const ValueKey('search-filters-toggle'),
                  on: s.filtersOpen || !s.searchFilters.isEmpty,
                  tip: t('Filtres de recherche'),
                  onTap: s.toggleFiltersBar,
                ),
                if (hasQ || !s.searchFilters.isEmpty)
                  Hov(
                    onTap: s.clearQuery,
                    tip: t('Effacer la recherche'),
                    builder: (_, _) => Padding(
                      padding: const EdgeInsets.all(2),
                      child: Ico('x', color: p.dim),
                    ),
                  ),
              ],
            ),
          ),
          if (!mobile) const Spacer(),
          SizedBox(width: mobile ? 6 : 12),
          Builder(
            builder: (ctx) => Hov(
              onTap: () => showAccountPop(ctx),
              tip: t('Changer de compte'),
              builder: (_, h) => AnimatedContainer(
                duration: const Duration(milliseconds: 130),
                padding: EdgeInsets.symmetric(
                  horizontal: wide ? 12 : 8,
                  vertical: 7,
                ),
                decoration: BoxDecoration(
                  color: h ? p.surface : null,
                  border: Border.all(color: h ? p.dim : p.line),
                ),
                child: Row(
                  mainAxisSize: MainAxisSize.min,
                  children: [
                    s.unified
                        ? Container(
                            width: 22,
                            height: 22,
                            alignment: Alignment.center,
                            color: p.accent,
                            child: Ico('inbox', size: 14, color: p.onAccent),
                          )
                        : AccountAvatar(s.active, size: 22),
                    if (wide) ...[
                      const SizedBox(width: 10),
                      ConstrainedBox(
                        constraints: const BoxConstraints(maxWidth: 190),
                        child: Text(
                          s.unified
                              ? t('Toutes les boîtes')
                              : (s.active.name.trim().isEmpty
                                    ? s.active.email
                                    : s.active.name),
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                          style: Fonts.sans(
                            size: 13.5,
                            color: h ? p.text : p.muted,
                            height: 1.2,
                          ),
                        ),
                      ),
                    ],
                    const SizedBox(width: 8),
                    Ico('chev', size: 14, color: h ? p.text : p.muted),
                  ],
                ),
              ),
            ),
          ),
          SizedBox(width: mobile ? 6 : 12),
          Btn(
            icon: 'bot',
            label: t('Assistant IA'),
            iconOnly: mobile,
            on: s.view == PaneView.assistant,
            onTap: s.toggleAssistant,
            padding: mobile ? const EdgeInsets.symmetric(vertical: 10) : null,
            tip: t('Assistant IA'),
          ),
          SizedBox(width: mobile ? 6 : 12),
          Btn(
            icon: 'gear',
            label: t('Paramètres'),
            iconOnly: mobile,
            on: s.view == PaneView.settings,
            onTap: s.toggleSettings,
            padding: mobile ? const EdgeInsets.symmetric(vertical: 10) : null,
            tip: t('Paramètres'),
          ),
        ],
      ),
    );
  }
}
