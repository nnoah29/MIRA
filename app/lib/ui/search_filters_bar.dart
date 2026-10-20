import 'package:flutter/material.dart';
import 'package:provider/provider.dart';

import '../i18n/i18n.dart';
import '../state/search_filters.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import '../theme/icons.dart';
import 'overlays.dart';
import 'widgets.dart';

String _periodName(SearchPeriod p) => switch (p) {
  SearchPeriod.any => t('Toute période'),
  SearchPeriod.day => t('Dernières 24 h'),
  SearchPeriod.week => t('7 derniers jours'),
  SearchPeriod.month => t('30 derniers jours'),
};

/// Pastilles de filtres de la recherche : s'ajoutent au texte tapé, ou le remplacent.
class SearchFiltersBar extends StatefulWidget {
  final double padding;
  const SearchFiltersBar({super.key, required this.padding});

  @override
  State<SearchFiltersBar> createState() => _SearchFiltersBarState();
}

class _SearchFiltersBarState extends State<SearchFiltersBar> {
  final from = TextEditingController();

  @override
  void dispose() {
    from.dispose();
    super.dispose();
  }

  void _set(MailStore s, SearchFilters f) => s.setSearchFilters(f);

  @override
  Widget build(BuildContext context) {
    final s = context.watch<MailStore>();
    final f = s.searchFilters;
    if (f.from.isEmpty && from.text.isNotEmpty) from.clear();
    return Padding(
      padding: EdgeInsets.fromLTRB(widget.padding, 12, widget.padding, 0),
      child: Wrap(
        spacing: 6,
        runSpacing: 6,
        crossAxisAlignment: WrapCrossAlignment.center,
        children: [
          _Toggle(
            id: 'unread',
            label: t('Non lus'),
            on: f.unread,
            onTap: () => _set(s, f.copyWith(unread: !f.unread)),
          ),
          _Toggle(
            id: 'starred',
            label: t('Favoris'),
            on: f.starred,
            onTap: () => _set(s, f.copyWith(starred: !f.starred)),
          ),
          _Toggle(
            id: 'attachments',
            label: t('Pièces jointes'),
            on: f.attachments,
            onTap: () => _set(s, f.copyWith(attachments: !f.attachments)),
          ),
          _FromField(
            controller: from,
            on: f.from.trim().isNotEmpty,
            onChanged: (v) => _set(s, f.copyWith(from: v)),
          ),
          Builder(
            builder: (ctx) => _Toggle(
              id: 'period',
              label: f.period == SearchPeriod.any
                  ? t('Période')
                  : _periodName(f.period),
              on: f.period != SearchPeriod.any,
              caret: true,
              onTap: () => _periodPop(ctx),
            ),
          ),
          if (s.accounts.length > 1)
            Builder(
              builder: (ctx) => _Toggle(
                id: 'account',
                label:
                    s.accounts
                        .where((a) => a.id == f.account)
                        .map((a) => a.name.isEmpty ? a.email : a.name)
                        .firstOrNull ??
                    t('Compte'),
                on: f.account != null,
                caret: true,
                onTap: () => _accountPop(ctx),
              ),
            ),
          if (s.labelDefs.isNotEmpty)
            Builder(
              builder: (ctx) => _Toggle(
                id: 'label',
                label: f.label == null ? t('Libellé') : s.labelTitle(f.label!),
                on: f.label != null,
                caret: true,
                onTap: () => _labelPop(ctx),
              ),
            ),
          if (!f.isEmpty)
            Hov(
              key: const ValueKey('filter-reset'),
              onTap: () => _set(s, const SearchFilters()),
              builder: (_, h) => Padding(
                padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 6),
                child: Text(
                  t('Réinitialiser'),
                  style: Fonts.mono(
                    size: 11,
                    color: h ? context.pal.text : context.pal.muted,
                    ls: 0,
                    height: 1.5,
                  ),
                ),
              ),
            ),
        ],
      ),
    );
  }

  void _periodPop(BuildContext anchor) =>
      Pops.show(anchor, 'search-period', (ctx, close) {
        final s = ctx.read<MailStore>();
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          mainAxisSize: MainAxisSize.min,
          children: [
            for (final p in SearchPeriod.values)
              PopItem(
                key: ValueKey('period-${p.name}'),
                leading: const SizedBox.shrink(),
                label: _periodName(p),
                on: s.searchFilters.period == p,
                onTap: () {
                  close();
                  s.setSearchFilters(s.searchFilters.copyWith(period: p));
                },
              ),
          ],
        );
      });

  void _accountPop(BuildContext anchor) =>
      Pops.show(anchor, 'search-account', (ctx, close) {
        final s = ctx.read<MailStore>();
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          mainAxisSize: MainAxisSize.min,
          children: [
            PopItem(
              leading: const SizedBox.shrink(),
              label: t('Tous les comptes'),
              on: s.searchFilters.account == null,
              onTap: () {
                close();
                s.setSearchFilters(s.searchFilters.copyWith(account: null));
              },
            ),
            for (final a in s.accounts)
              PopItem(
                key: ValueKey('account-${a.id}'),
                leading: const SizedBox.shrink(),
                label: a.name.isEmpty ? a.email : a.name,
                on: s.searchFilters.account == a.id,
                onTap: () {
                  close();
                  s.setSearchFilters(s.searchFilters.copyWith(account: a.id));
                },
              ),
          ],
        );
      });

  void _labelPop(BuildContext anchor) =>
      Pops.show(anchor, 'search-label', (ctx, close) {
        final s = ctx.read<MailStore>();
        return Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          mainAxisSize: MainAxisSize.min,
          children: [
            PopItem(
              leading: const SizedBox.shrink(),
              label: t('Tous les libellés'),
              on: s.searchFilters.label == null,
              onTap: () {
                close();
                s.setSearchFilters(s.searchFilters.copyWith(label: null));
              },
            ),
            for (final e in s.labelDefs.entries)
              PopItem(
                key: ValueKey('label-${e.key}'),
                leading: Square(10, Color(e.value.c)),
                label: s.labelTitle(e.key),
                on: s.searchFilters.label == e.key,
                onTap: () {
                  close();
                  s.setSearchFilters(s.searchFilters.copyWith(label: e.key));
                },
              ),
          ],
        );
      });
}

/// Pastille à bascule, avec une flèche quand elle ouvre un choix.
class _Toggle extends StatelessWidget {
  final String id, label;
  final bool on, caret;
  final VoidCallback onTap;
  const _Toggle({
    required this.id,
    required this.label,
    required this.on,
    required this.onTap,
    this.caret = false,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Hov(
      key: ValueKey('filter-$id'),
      onTap: onTap,
      builder: (_, h) {
        final fg = on ? p.accent : (h ? p.text : p.muted);
        return AnimatedContainer(
          duration: const Duration(milliseconds: 130),
          padding: const EdgeInsets.symmetric(horizontal: 11, vertical: 6.25),
          decoration: BoxDecoration(
            color: on ? p.accentSoft : null,
            border: Border.all(color: on ? p.accent : p.line),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Text(
                label,
                style: Fonts.mono(size: 11, color: fg, ls: 0, height: 1.5),
              ),
              if (caret) ...[
                const SizedBox(width: 6),
                Ico('chev', size: 11, color: fg),
              ],
            ],
          ),
        );
      },
    );
  }
}

/// Champ « de : … » : filtre sur le nom ou l'adresse de l'expéditeur.
class _FromField extends StatelessWidget {
  final TextEditingController controller;
  final bool on;
  final ValueChanged<String> onChanged;
  const _FromField({
    required this.controller,
    required this.on,
    required this.onChanged,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Container(
      width: 150,
      padding: const EdgeInsets.symmetric(horizontal: 11, vertical: 6.25),
      decoration: BoxDecoration(
        color: on ? p.accentSoft : null,
        border: Border.all(color: on ? p.accent : p.line),
      ),
      child: TextField(
        key: const ValueKey('filter-from'),
        controller: controller,
        onChanged: onChanged,
        cursorColor: p.accent,
        cursorWidth: 1,
        style: Fonts.mono(
          size: 11,
          color: on ? p.accent : p.text,
          ls: 0,
          height: 1.5,
        ),
        decoration: InputDecoration(
          isDense: true,
          border: InputBorder.none,
          contentPadding: EdgeInsets.zero,
          hintText: t('De : expéditeur'),
          hintStyle: Fonts.mono(size: 11, color: p.muted, ls: 0, height: 1.5),
        ),
      ),
    );
  }
}
