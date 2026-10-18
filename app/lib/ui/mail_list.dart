import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../data/format.dart';
import '../data/html_lite.dart';
import '../data/models.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import '../theme/icons.dart';
import 'overlays.dart';
import 'widgets.dart';
import '../i18n/i18n.dart';

class MailsColumn extends StatelessWidget {
  const MailsColumn({super.key});

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    final p = context.pal;
    final mobile = MediaQuery.sizeOf(context).width <= 760;
    final hp = mobile ? 16.0 : 24.0;
    final title = s.label != null
        ? s.labelTitle(s.label!)
        : t(folderById(s.folder).n);
    final sc = s.scope();
    final un = s.nonRead(sc);
    final sub = un > 0
        ? tn(un, 'non lu', 'non lus')
        : tn(sc.length, 'message', 'messages');
    int cnt(String k) => sc.where((m) => m.flags.contains(k)).length;
    final pills = [
      ('all', t('Tous')),
      ('needs', t('À répondre')),
      ('waiting', t('En attente')),
      ('mention', t('Mentions')),
    ];
    final vis = s.visible();
    final nSel = vis.where((m) => s.sel.contains(m.id)).length;
    final st = nSel == 0 ? 0 : (nSel == vis.length ? 2 : 1);

    return Container(
      decoration: BoxDecoration(
        border: Border(right: BorderSide(color: p.lineSoft)),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: EdgeInsets.fromLTRB(hp, 24, hp, 0),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          Flexible(
                            child: Text(
                              title.toUpperCase(),
                              maxLines: 1,
                              overflow: TextOverflow.ellipsis,
                              style: Fonts.display(size: 34, color: p.text),
                            ),
                          ),
                          const Padding(
                            padding: EdgeInsets.only(left: 6, top: 22),
                            child: BlinkingUnderscore(),
                          ),
                        ],
                      ),
                      const SizedBox(height: 6),
                      Mono(sub),
                    ],
                  ),
                ),
                if (s.source != null) ...[
                  IBtn(
                    'refresh',
                    on: s.syncing,
                    tip: t('Actualiser (F5)'),
                    onTap: s.refreshNow,
                  ),
                  const SizedBox(width: 6),
                ],
                Builder(
                  builder: (ctx) => IBtn(
                    'filter',
                    tip: t('Trier'),
                    onTap: () => showSortPop(ctx),
                  ),
                ),
                const SizedBox(width: 6),
                IBtn(
                  'pen',
                  tip: t('Nouveau message (c)'),
                  onTap: () => s.openCompose(const ComposeInit()),
                ),
              ],
            ),
          ),
          Padding(
            padding: EdgeInsets.fromLTRB(hp, 18, hp, 14),
            child: Wrap(
              spacing: 6,
              runSpacing: 6,
              children: [
                for (final (k, n) in pills)
                  Hov(
                    onTap: () => s.setPill(k),
                    builder: (_, h) {
                      final on = s.pill == k;
                      final fg = on ? p.accent : (h ? p.text : p.muted);
                      return AnimatedContainer(
                        duration: const Duration(milliseconds: 130),
                        padding: const EdgeInsets.symmetric(
                          horizontal: 11,
                          vertical: 6.25,
                        ),
                        decoration: BoxDecoration(
                          color: on ? p.accentSoft : null,
                          border: Border.all(color: on ? p.accent : p.line),
                        ),
                        child: Row(
                          mainAxisSize: MainAxisSize.min,
                          children: [
                            Text(
                              n,
                              style: Fonts.mono(
                                size: 11,
                                color: fg,
                                ls: 0,
                                height: 1.5,
                              ),
                            ),
                            if (k != 'all') ...[
                              const SizedBox(width: 6),
                              Text(
                                '${cnt(k)}',
                                style: Fonts.mono(
                                  size: 11,
                                  w: FontWeight.w500,
                                  color: on ? p.accent : p.dim,
                                  ls: 0,
                                  height: 1.5,
                                ),
                              ),
                            ],
                          ],
                        ),
                      );
                    },
                  ),
              ],
            ),
          ),
          // barre d'actions groupées
          Container(
            padding: EdgeInsets.symmetric(horizontal: hp, vertical: 8),
            decoration: BoxDecoration(
              border: Border.symmetric(
                horizontal: BorderSide(color: p.lineSoft),
              ),
            ),
            child: Row(
              children: [
                Hov(
                  onTap: s.selAll,
                  tip: t('Tout sélectionner'),
                  builder: (_, h) => Container(
                    margin: const EdgeInsets.only(right: 8),
                    padding: const EdgeInsets.symmetric(
                      horizontal: 9,
                      vertical: 5,
                    ),
                    decoration: BoxDecoration(
                      color: h ? p.surface : null,
                      border: Border.all(color: p.line),
                    ),
                    child: Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        _Bx(st),
                        const SizedBox(width: 8),
                        Text(
                          s.sel.isEmpty ? '' : '${s.sel.length}',
                          style: Fonts.mono(
                            size: 11,
                            w: FontWeight.w500,
                            color: h ? p.text : p.muted,
                            ls: 0,
                            height: 1.5,
                          ),
                        ),
                        const SizedBox(width: 8),
                        Ico('chev', size: 12, color: h ? p.text : p.muted),
                      ],
                    ),
                  ),
                ),
                _Ic('archive', t('Archiver (e)'), s.archive),
                const SizedBox(width: 4),
                _Ic('warn', t('Signaler comme spam'), s.spam),
                const SizedBox(width: 4),
                _Ic('trash', t('Supprimer (#)'), s.trash),
                const SizedBox(width: 4),
                _Ic('inbox', t('Marquer lu / non lu (u)'), s.toggleRead),
                const SizedBox(width: 4),
                Builder(
                  builder: (ctx) =>
                      _Ic('clock', t('Reporter'), () => showSnoozePop(ctx)),
                ),
                const Spacer(),
                Builder(
                  builder: (ctx) =>
                      _Ic('tag', t('Libellés'), () => showLabelPop(ctx)),
                ),
                const SizedBox(width: 4),
                Builder(
                  builder: (ctx) =>
                      _Ic('dots', t('Plus'), () => showMorePop(ctx)),
                ),
              ],
            ),
          ),
          const Expanded(child: MailListView()),
        ],
      ),
    );
  }
}

class _Bx extends StatelessWidget {
  final int st; // 0 aucun, 1 partiel, 2 tous
  const _Bx(this.st);
  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Container(
      width: 14,
      height: 14,
      alignment: Alignment.center,
      decoration: BoxDecoration(
        color: st == 2 ? p.accent : null,
        border: Border.all(color: st == 0 ? p.dim : p.accent),
      ),
      child: st == 1 ? Container(width: 8, height: 2, color: p.accent) : null,
    );
  }
}

class _Ic extends StatelessWidget {
  final String icon, tip;
  final VoidCallback onTap;
  const _Ic(this.icon, this.tip, this.onTap);
  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Hov(
      onTap: onTap,
      tip: tip,
      builder: (_, h) => Container(
        width: 30,
        height: 30,
        alignment: Alignment.center,
        color: h ? p.surface : null,
        child: Ico(icon, color: h ? p.text : p.muted),
      ),
    );
  }
}

class MailListView extends StatefulWidget {
  const MailListView({super.key});
  @override
  State<MailListView> createState() => _MailListViewState();
}

class _MailListViewState extends State<MailListView> {
  final ctl = ScrollController();
  final keys = <String, GlobalKey>{};
  int lastTick = -1;

  @override
  void dispose() {
    ctl.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    final p = context.pal;
    final l = s.visible();
    final mobile = MediaQuery.sizeOf(context).width <= 760;
    final hp = mobile ? 16.0 : 24.0;

    if (s.scrollTick != lastTick) {
      lastTick = s.scrollTick;
      final id = s.open;
      if (id != null) {
        WidgetsBinding.instance.addPostFrameCallback((_) {
          final c = keys[id]?.currentContext;
          if (c != null && c.mounted) {
            Scrollable.ensureVisible(
              c,
              alignmentPolicy: ScrollPositionAlignmentPolicy.keepVisibleAtEnd,
              duration: Duration.zero,
            );
          }
        });
      }
    }

    if (l.isEmpty) {
      return Padding(
        padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 56),
        child: Align(
          alignment: Alignment.topCenter,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Ico(
                s.q.isNotEmpty ? 'search' : 'inbox',
                size: 34,
                stroke: 1.2,
                color: p.dim,
              ),
              const SizedBox(height: 12),
              Mono(s.q.isNotEmpty ? t('Aucun résultat') : t('Rien ici')),
              Text(
                s.q.isNotEmpty
                    ? t('Aucun message ne correspond à « {0} ».', [s.q])
                    : t('Ce dossier est vide.'),
                textAlign: TextAlign.center,
                style: Fonts.sans(size: 15, color: p.dim),
              ),
            ],
          ),
        ),
      );
    }

    final groups = <String, List<Mail>>{};
    for (final m in l) {
      groups.putIfAbsent(groupOf(m.date), () => []).add(m);
    }
    return Scrollbar(
      controller: ctl,
      child: CustomScrollView(
        controller: ctl,
        // ignore: deprecated_member_use
        cacheExtent: 6000,
        slivers: [
          for (final e in groups.entries)
            SliverMainAxisGroup(
              slivers: [
                SliverPersistentHeader(
                  pinned: true,
                  delegate: _GrpHeader(t(e.key), hp, p),
                ),
                SliverList.list(
                  children: [
                    for (final (i, m) in e.value.indexed)
                      KeyedSubtree(
                        key: keys.putIfAbsent(m.id, GlobalKey.new),
                        child: FadeSlideIn(
                          delay: Duration(milliseconds: 28 * (i > 7 ? 7 : i)),
                          from: const Offset(0, 10),
                          child: MailRow(m, hp: hp),
                        ),
                      ),
                  ],
                ),
              ],
            ),
        ],
      ),
    );
  }
}

class _GrpHeader extends SliverPersistentHeaderDelegate {
  final String t;
  final double hp;
  final Pal p;
  _GrpHeader(this.t, this.hp, this.p);
  @override
  double get minExtent => 40.5;
  @override
  double get maxExtent => 40.5;
  @override
  Widget build(
    BuildContext context,
    double shrinkOffset,
    bool overlapsContent,
  ) => Container(
    color: p.bg,
    padding: EdgeInsets.fromLTRB(hp, 16, hp, 8),
    alignment: Alignment.centerLeft,
    child: Mono(t),
  );
  @override
  bool shouldRebuild(_GrpHeader o) => o.t != t || o.p != p || o.hp != hp;
}

class MailRow extends StatelessWidget {
  final Mail m;
  final double hp;
  const MailRow(this.m, {super.key, required this.hp});

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    final p = context.pal;
    final out = m.folder == 'sent' || m.folder == 'drafts';
    final who = out
        ? t('À : {0}', [
            m.to.isEmpty
                ? t('(aucun destinataire)')
                : m.to.map((x) => x.name).join(', '),
          ])
        : m.from.name;
    final avName = out ? (m.to.isNotEmpty ? m.to[0].name : '?') : m.from.name;
    final last = m.msgs.last;
    final hasAtt = m.msgs.any((x) => x.att.isNotEmpty);
    final isSel = s.open == m.id;
    final checked = s.sel.contains(m.id);
    final dense = s.dense;
    final read = !m.unread;
    final snipText = plainOf(last.body);
    final snip = snipText.length > 170 ? snipText.substring(0, 170) : snipText;
    String? tag;
    if (m.folder == 'drafts') {
      tag = t('Brouillon');
    } else if (m.folder == 'snoozed') {
      tag = t('Reporté · {0}', [t(m.snooze ?? '')]);
    } else if (m.sched != null) {
      tag = t('Programmé · {0}', [t(m.sched!)]);
    }
    final avSize = dense ? 30.0 : 38.0;

    Widget row = Hov(
      onTap: () => s.openMail(m.id),
      builder: (_, h) {
        final bg = isSel ? p.surface2 : (h ? p.surface : Colors.transparent);
        final ring = isSel ? p.surface2 : (h ? p.surface : p.bg);
        return AnimatedContainer(
          duration: const Duration(milliseconds: 130),
          padding: EdgeInsets.fromLTRB(hp, dense ? 8 : 14, hp, dense ? 8 : 14),
          decoration: BoxDecoration(
            color: bg,
            border: Border(
              left: BorderSide(
                color: isSel ? p.accent : Colors.transparent,
                width: 2,
              ),
              bottom: BorderSide(color: p.lineSoft),
            ),
          ),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Padding(
                padding: EdgeInsets.only(top: dense ? 8 : 12, right: 2),
                child: Hov(
                  onTap: () => s.toggleSel(m.id),
                  builder: (_, _) => AnimatedContainer(
                    duration: const Duration(milliseconds: 120),
                    width: 14,
                    height: 14,
                    alignment: Alignment.center,
                    decoration: BoxDecoration(
                      color: checked ? p.accent : null,
                      border: Border.all(color: checked ? p.accent : p.dim),
                    ),
                    child: AnimatedScale(
                      scale: checked ? 1 : 0,
                      duration: const Duration(milliseconds: 160),
                      curve: Curves.easeOutBack,
                      child: Ico(
                        'check',
                        size: 10,
                        stroke: 3,
                        color: p.onAccent,
                      ),
                    ),
                  ),
                ),
              ),
              const SizedBox(width: 12),
              Stack(
                clipBehavior: Clip.none,
                children: [
                  Avatar(avName, size: avSize, font: dense ? 10 : 11),
                  if (!read)
                    Positioned(
                      top: -4,
                      right: -4,
                      child: Container(
                        width: 9,
                        height: 9,
                        decoration: BoxDecoration(
                          color: p.accent,
                          boxShadow: [BoxShadow(color: ring, spreadRadius: 2)],
                        ),
                      ),
                    ),
                ],
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Expanded(
                          child: Row(
                            children: [
                              Flexible(
                                child: Text(
                                  who,
                                  maxLines: 1,
                                  overflow: TextOverflow.ellipsis,
                                  style: Fonts.sans(
                                    size: 15,
                                    w: read ? FontWeight.w400 : FontWeight.w600,
                                    color: read ? p.muted : p.text,
                                  ),
                                ),
                              ),
                              if (s.unified) ...[
                                const SizedBox(width: 8),
                                Tooltip(
                                  message: accountById(m.account)?.email ?? '',
                                  child: Container(
                                    padding: const EdgeInsets.symmetric(
                                      horizontal: 5,
                                    ),
                                    decoration: BoxDecoration(
                                      border: Border.all(color: p.line),
                                    ),
                                    child: Text(
                                      (accountById(m.account)?.email ?? '')
                                          .split('@')
                                          .first
                                          .toUpperCase(),
                                      maxLines: 1,
                                      style: Fonts.mono(
                                        size: 9.5,
                                        color: p.dim,
                                        ls: .08,
                                        height: 1.5,
                                      ),
                                    ),
                                  ),
                                ),
                              ],
                              if (m.msgs.length > 1) ...[
                                const SizedBox(width: 8),
                                Container(
                                  padding: const EdgeInsets.symmetric(
                                    horizontal: 5,
                                  ),
                                  decoration: BoxDecoration(
                                    border: Border.all(color: p.line),
                                  ),
                                  child: Text(
                                    '${m.msgs.length}',
                                    style: Fonts.mono(
                                      size: 10,
                                      w: FontWeight.w500,
                                      color: p.dim,
                                      ls: 0,
                                      height: 1.4,
                                    ),
                                  ),
                                ),
                              ],
                            ],
                          ),
                        ),
                        const SizedBox(width: 8),
                        Text(
                          rowTime(m.date),
                          style: Fonts.mono(
                            size: 11,
                            w: FontWeight.w500,
                            color: read ? p.dim : p.accent,
                            ls: .06,
                            height: 1.5,
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 1),
                    Row(
                      children: [
                        Expanded(
                          child: Text.rich(
                            TextSpan(
                              children: [
                                if (tag != null)
                                  WidgetSpan(
                                    alignment: PlaceholderAlignment.middle,
                                    child: Container(
                                      margin: const EdgeInsets.only(right: 8),
                                      padding: const EdgeInsets.symmetric(
                                        horizontal: 6,
                                        vertical: 1,
                                      ),
                                      decoration: BoxDecoration(
                                        border: Border.all(color: p.accent),
                                      ),
                                      child: Text(
                                        tag.toUpperCase(),
                                        style: Fonts.mono(
                                          size: 10.5,
                                          color: p.accent,
                                          ls: .12,
                                          height: 1.3,
                                        ),
                                      ),
                                    ),
                                  ),
                                TextSpan(text: m.subject),
                              ],
                            ),
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: Fonts.sans(
                              size: 15,
                              w: read ? FontWeight.w400 : FontWeight.w500,
                              color: read ? p.muted : p.text,
                            ),
                          ),
                        ),
                        const SizedBox(width: 8),
                        if (hasAtt) ...[
                          Ico('clip', size: 13, color: p.dim),
                          const SizedBox(width: 8),
                        ],
                        for (final lb in m.labels.take(2))
                          if (s.labelDef(lb) != null) ...[
                            Square(8, Color(s.labelDef(lb)!.c)),
                            const SizedBox(width: 8),
                          ],
                        _Star(m, h),
                      ],
                    ),
                    if (!dense && s.snip)
                      Text(
                        snip,
                        maxLines: 2,
                        overflow: TextOverflow.ellipsis,
                        style: Fonts.sans(
                          size: 13.5,
                          color: p.dim,
                          height: 1.45,
                        ),
                      ),
                  ],
                ),
              ),
            ],
          ),
        );
      },
    );
    if (s.freshId == m.id) {
      row = TweenAnimationBuilder<double>(
        key: ValueKey('fresh-${m.id}'),
        tween: Tween(begin: 0, end: 1),
        duration: const Duration(milliseconds: 500),
        curve: Curves.easeOut,
        builder: (_, t, c) => Opacity(
          opacity: .2 + .8 * t,
          child: Transform.translate(offset: Offset(0, -6 * (1 - t)), child: c),
        ),
        child: row,
      );
    }
    return row;
  }
}

class _Star extends StatelessWidget {
  final Mail m;
  final bool rowHover;
  const _Star(this.m, this.rowHover);
  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final s = context.read<MailStore>();
    return Hov(
      onTap: () => s.toggleStar(m.id),
      builder: (_, _) => AnimatedOpacity(
        duration: const Duration(milliseconds: 140),
        opacity: rowHover || m.starred ? 1 : 0,
        child: Transform.translate(
          offset: const Offset(2, 0),
          child: Container(
            width: 18,
            height: 18,
            alignment: Alignment.center,
            child: Bounce(
              trigger: m.starred,
              child: Ico(
                'star',
                size: 13,
                color: m.starred ? p.accent : p.dim,
                filled: m.starred,
              ),
            ),
          ),
        ),
      ),
    );
  }
}

/// Tiret du bas « _ » orange qui clignote, comme un curseur de terminal.
class BlinkingUnderscore extends StatefulWidget {
  const BlinkingUnderscore({super.key});
  @override
  State<BlinkingUnderscore> createState() => _BlinkingUnderscoreState();
}

class _BlinkingUnderscoreState extends State<BlinkingUnderscore>
    with SingleTickerProviderStateMixin {
  late final AnimationController _c = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 1060),
  )..repeat();

  @override
  void dispose() {
    _c.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return AnimatedBuilder(
      animation: _c,
      // Allumé la première moitié du cycle, éteint la seconde (clignotement net).
      builder: (_, _) => Opacity(
        opacity: _c.value < .5 ? 1 : 0,
        child: Container(width: 18, height: 4, color: p.accent),
      ),
    );
  }
}
