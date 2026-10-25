import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

import '../data/format.dart';
import '../i18n/i18n.dart';
import '../state/store.dart';
import '../theme/app_theme.dart';
import 'overlays.dart';
import 'widgets.dart';

const _cellW = 36.0;
const _cellH = 32.0;

/// Premier instant proposé : dans une heure, arrondi aux 5 minutes supérieures.
DateTime defaultSendTime([DateTime? from]) {
  final t = (from ?? DateTime.now()).add(const Duration(hours: 1));
  final m = (t.minute / 5).ceil() * 5;
  return DateTime(t.year, t.month, t.day, t.hour).add(Duration(minutes: m));
}

/// Popover de programmation à date et heure précises (calendrier du mois, heure, minutes).
void showSendTimePop(BuildContext anchor, MailStore s) {
  Pops.show(
    anchor,
    'sched-at',
    (ctx, close) => SendTimePicker(
      onPick: (at) {
        close();
        s.send(fullDate(at.millisecondsSinceEpoch), at.millisecondsSinceEpoch);
      },
    ),
    maxWidth: 300,
  );
}

class SendTimePicker extends StatefulWidget {
  final ValueChanged<DateTime> onPick;
  final DateTime? initial;
  const SendTimePicker({super.key, required this.onPick, this.initial});

  @override
  State<SendTimePicker> createState() => _SendTimePickerState();
}

class _SendTimePickerState extends State<SendTimePicker> {
  late DateTime at = widget.initial ?? defaultSendTime();
  late DateTime month = DateTime(at.year, at.month);

  bool get valid => at.isAfter(DateTime.now());

  void _day(DateTime d) =>
      setState(() => at = DateTime(d.year, d.month, d.day, at.hour, at.minute));

  void _shift({int hours = 0, int minutes = 0}) {
    final h = (at.hour + hours) % 24;
    final m = (at.minute + minutes) % 60;
    setState(() => at = DateTime(at.year, at.month, at.day, h, m));
  }

  void _month(int delta) =>
      setState(() => month = DateTime(month.year, month.month + delta));

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final loc = appLang.dateLocale;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      mainAxisSize: MainAxisSize.min,
      children: [
        PopTitle(t('Programmer l’envoi')),
        _header(p, DateFormat('MMMM y', loc).format(month)),
        _weekdays(loc),
        _grid(p),
        const PopSep(),
        _timeRow(p),
        const PopSep(),
        _footer(p, loc),
      ],
    );
  }

  Widget _header(Pal p, String label) {
    final first = DateTime.now();
    final canBack = month.isAfter(DateTime(first.year, first.month));
    return Row(
      children: [
        Opacity(
          opacity: canBack ? 1 : .3,
          child: IBtn(
            'arrowl',
            size: 14,
            box: 30,
            onTap: canBack ? () => _month(-1) : null,
          ),
        ),
        Expanded(
          child: Center(
            child: Mono(label, size: 11, color: p.text, height: null),
          ),
        ),
        Transform.flip(
          flipX: true,
          child: IBtn('arrowl', size: 14, box: 30, onTap: () => _month(1)),
        ),
      ],
    );
  }

  Widget _weekdays(String loc) => Row(
    children: [
      for (var i = 0; i < 7; i++)
        SizedBox(
          width: _cellW,
          height: 24,
          child: Center(
            child: Mono(
              DateFormat('EEEEE', loc).format(DateTime(2024, 1, 1 + i)),
              size: 10,
              height: null,
            ),
          ),
        ),
    ],
  );

  Widget _grid(Pal p) {
    final lead = DateTime(month.year, month.month).weekday - 1;
    final days = DateUtils.getDaysInMonth(month.year, month.month);
    final rows = ((lead + days) / 7).ceil();
    return Column(
      children: [
        for (var r = 0; r < rows; r++)
          Row(
            children: [
              for (var c = 0; c < 7; c++) _cell(p, r * 7 + c - lead + 1, days),
            ],
          ),
      ],
    );
  }

  Widget _cell(Pal p, int n, int days) {
    if (n < 1 || n > days) return const SizedBox(width: _cellW, height: _cellH);
    final d = DateTime(month.year, month.month, n);
    final now = DateTime.now();
    final past = d.isBefore(DateTime(now.year, now.month, now.day));
    final sel = DateUtils.isSameDay(d, at);
    final today = DateUtils.isSameDay(d, now);
    return Hov(
      onTap: past ? null : () => _day(d),
      builder: (_, h) => Container(
        width: _cellW,
        height: _cellH,
        alignment: Alignment.center,
        decoration: BoxDecoration(
          color: sel ? p.accent : (h && !past ? p.surface : null),
          border: today && !sel ? Border.all(color: p.accent) : null,
        ),
        child: Text(
          '$n',
          style: Fonts.sans(
            size: 13,
            color: sel
                ? p.onAccent
                : (past ? p.dim.withValues(alpha: .45) : p.text),
            w: sel ? FontWeight.w700 : FontWeight.w400,
          ),
        ),
      ),
    );
  }

  Widget _timeRow(Pal p) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 2),
    child: Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        _stepper(
          p,
          pad2(at.hour),
          () => _shift(hours: -1),
          () => _shift(hours: 1),
        ),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 8),
          child: Mono(':', size: 18, color: p.dim, height: null),
        ),
        _stepper(
          p,
          pad2(at.minute),
          () => _shift(minutes: -5),
          () => _shift(minutes: 5),
        ),
      ],
    ),
  );

  Widget _stepper(Pal p, String v, VoidCallback less, VoidCallback more) => Row(
    children: [
      IBtn('minus', size: 14, box: 30, onTap: less),
      SizedBox(
        width: 52,
        child: Center(
          child: Text(
            v,
            softWrap: false,
            style: Fonts.mono(
              size: 22,
              w: FontWeight.w600,
              color: p.text,
              ls: 0,
              height: null,
            ),
          ),
        ),
      ),
      IBtn('plus', size: 14, box: 30, onTap: more),
    ],
  );

  Widget _footer(Pal p, String loc) => Padding(
    padding: const EdgeInsets.fromLTRB(6, 6, 6, 4),
    child: Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Padding(
          padding: const EdgeInsets.only(bottom: 10),
          child: Text(
            DateFormat('EEEE d MMMM, HH:mm', loc).format(at),
            textAlign: TextAlign.center,
            style: Fonts.sans(size: 13, color: valid ? p.muted : p.accent),
          ),
        ),
        Opacity(
          opacity: valid ? 1 : .4,
          child: IgnorePointer(
            ignoring: !valid,
            child: Btn(
              icon: 'clock',
              label: t('Programmer'),
              go: true,
              onTap: () => widget.onPick(at),
            ),
          ),
        ),
      ],
    ),
  );
}
