import 'package:flutter/material.dart';

import '../data/models.dart';
import '../theme/app_theme.dart';

/// Bits de style par caractère.
class Fx {
  static const bold = 1; // <b> posé par l'utilisateur (gras 700)
  static const italic = 2;
  static const underline = 4;
  static const code = 8;
  static const at = 16;
  static const link = 32;
  static const strong = 64; // <strong> du contenu reçu (500, couleur texte)
  static const gap =
      128; // saut de paragraphe : ligne vide de 16 px (marge d'un <p>)
  static const inheritMask = bold | italic | underline;
}

/// Éditeur de texte riche minimal : un texte + un masque de styles par caractère.
class RichController extends TextEditingController {
  List<int> flags = [];
  int? pending; // style forcé pour la prochaine saisie (curseur réduit)
  Color baseColor = Colors.white;
  Color strongColor = Colors.white;
  Color accent = Colors.orange;
  Color codeBg = Colors.transparent;
  VoidCallback? onUserChange;

  void setContent(String t, List<int> f) {
    flags = List<int>.from(f);
    pending = null;
    super.value = TextEditingValue(
      text: t,
      selection: TextSelection.collapsed(offset: 0),
    );
  }

  @override
  set value(TextEditingValue nv) {
    final old = text;
    final nt = nv.text;
    if (old != nt) {
      var pre = 0;
      final minLen = old.length < nt.length ? old.length : nt.length;
      while (pre < minLen && old.codeUnitAt(pre) == nt.codeUnitAt(pre)) {
        pre++;
      }
      var suf = 0;
      while (suf < minLen - pre &&
          old.codeUnitAt(old.length - 1 - suf) ==
              nt.codeUnitAt(nt.length - 1 - suf)) {
        suf++;
      }
      final removed = old.length - pre - suf;
      final inserted = nt.length - pre - suf;
      final base =
          pending ??
          (pre > 0 && pre - 1 < flags.length
              ? flags[pre - 1] & Fx.inheritMask
              : 0);
      flags.replaceRange(pre, pre + removed, List<int>.filled(inserted, base));
      pending = null;
    } else if (nv.selection != value.selection) {
      pending = null;
    }
    super.value = nv;
  }

  /// Le style [bit] est-il actif sur toute la sélection (ou sous le curseur) ?
  bool isOn(int bit) {
    final s = selection;
    if (!s.isValid) return false;
    if (s.isCollapsed) {
      if (pending != null) return pending! & bit != 0;
      final i = s.start - 1;
      return i >= 0 && i < flags.length && flags[i] & bit != 0;
    }
    for (var i = s.start; i < s.end && i < flags.length; i++) {
      if (flags[i] & bit == 0) return false;
    }
    return true;
  }

  void toggle(int bit) {
    final s = selection;
    if (!s.isValid) return;
    final on = isOn(bit);
    if (s.isCollapsed) {
      final i = s.start - 1;
      final cur =
          pending ??
          (i >= 0 && i < flags.length ? flags[i] & Fx.inheritMask : 0);
      pending = on ? cur & ~bit : cur | bit;
    } else {
      for (var i = s.start; i < s.end && i < flags.length; i++) {
        var f = flags[i];
        if (bit == Fx.bold) f &= ~Fx.strong;
        flags[i] = on ? f & ~bit : f | bit;
      }
    }
    notifyListeners();
  }

  /// Liste à puces : préfixe chaque ligne de la sélection par « • ».
  void toggleBullets() {
    final s = selection;
    if (!s.isValid) return;
    final t = text;
    var a = s.start == 0 ? 0 : t.lastIndexOf('\n', s.start - 1) + 1;
    var bEnd = t.indexOf('\n', s.end);
    if (bEnd == -1) bEnd = t.length;
    final seg = t.substring(a, bEnd);
    final lines = seg.split('\n');
    final all = lines.every((l) => l.startsWith('• '));
    final newLines = lines.map((l) => all ? l.substring(2) : '• $l').toList();
    final nseg = newLines.join('\n');
    final f = <int>[];
    var pos = a;
    for (var i = 0; i < lines.length; i++) {
      if (all) {
        f.addAll(flags.sublist(pos + 2, pos + lines[i].length));
      } else {
        f.add(0);
        f.add(0);
        f.addAll(flags.sublist(pos, pos + lines[i].length));
      }
      if (i < lines.length - 1) f.add(0);
      pos += lines[i].length + 1;
    }
    flags.replaceRange(a, bEnd, f);
    pending = null;
    super.value = TextEditingValue(
      text: t.replaceRange(a, bEnd, nseg),
      selection: TextSelection(baseOffset: a, extentOffset: a + nseg.length),
    );
  }

  /// Marque en « saut de paragraphe » le 2e `\n` de chaque `\n\n` d'un texte programmatique.
  static List<int> gapFlags(String t) {
    final f = List<int>.filled(t.length, 0);
    for (var i = 1; i < t.length; i++) {
      if (t.codeUnitAt(i) == 10 &&
          t.codeUnitAt(i - 1) == 10 &&
          f[i - 1] & Fx.gap == 0) {
        f[i] = Fx.gap;
      }
    }
    return f;
  }

  TextStyle _styleFor(int f, TextStyle base) {
    var st = base;
    if (f & Fx.gap != 0) return st.copyWith(height: 1.0);
    if (f & Fx.strong != 0) {
      st = Fonts.sansWeight(st, FontWeight.w500).copyWith(color: strongColor);
    }
    if (f & Fx.bold != 0) st = Fonts.sansWeight(st, FontWeight.w700);
    if (f & Fx.italic != 0) st = st.copyWith(fontStyle: FontStyle.italic);
    final deco = <TextDecoration>[];
    if (f & Fx.underline != 0 || f & Fx.link != 0) {
      deco.add(TextDecoration.underline);
    }
    if (deco.isNotEmpty) {
      st = st.copyWith(
        decoration: TextDecoration.combine(deco),
        decorationColor: st.color,
      );
    }
    if (f & (Fx.at | Fx.link) != 0) {
      st = st.copyWith(color: accent, decorationColor: accent);
    }
    if (f & Fx.code != 0) {
      st = st.copyWith(
        fontFamily: 'monospace',
        fontSize: (st.fontSize ?? 16) * .9,
        backgroundColor: codeBg,
      );
    }
    return st;
  }

  @override
  TextSpan buildTextSpan({
    required BuildContext context,
    TextStyle? style,
    required bool withComposing,
  }) {
    final base = style ?? const TextStyle();
    final t = text;
    if (t.isEmpty) return TextSpan(style: base, text: '');
    final kids = <InlineSpan>[];
    var start = 0;
    for (var i = 1; i <= t.length; i++) {
      final cur = i < t.length ? (i < flags.length ? flags[i] : 0) : -1;
      final prev = start < flags.length ? flags[start] : 0;
      if (cur != prev) {
        kids.add(
          TextSpan(text: t.substring(start, i), style: _styleFor(prev, base)),
        );
        start = i;
      }
    }
    return TextSpan(style: base, children: kids);
  }

  // ----- conversions -----

  /// Blocs du message à partir du texte (une ligne vide « saut de paragraphe » sépare deux `<p>`).
  List<Block> toBlocks() {
    final lines = text.split('\n');
    var end = lines.length;
    while (end > 0 && lines[end - 1].isEmpty) {
      end--;
    }
    final starts = <int>[];
    var offset = 0;
    for (final l in lines) {
      starts.add(offset);
      offset += l.length + 1;
    }
    final out = <Block>[];
    List<Span>? cur;
    var normal = 0;
    var sawGap = false;
    for (var i = 0; i < end; i++) {
      if (lines[i].isEmpty) {
        final fl = starts[i] < flags.length ? flags[starts[i]] : 0;
        if (fl & Fx.gap != 0) {
          sawGap = true;
        } else {
          normal++;
        }
        continue;
      }
      if (cur != null && normal == 0 && !sawGap) {
        cur.add(const Span('\n'));
      } else {
        final empties = cur == null ? normal : (sawGap ? normal : normal - 1);
        if (cur != null) out.add(Para(cur));
        for (var k = 0; k < empties; k++) {
          out.add(const Para([]));
        }
        cur = [];
      }
      normal = 0;
      sawGap = false;
      cur.addAll(_spansOf(starts[i], lines[i]));
    }
    if (cur != null) out.add(Para(cur));
    return out;
  }

  List<Span> _spansOf(int start, String line) {
    final spans = <Span>[];
    var a = 0;
    for (var i = 1; i <= line.length; i++) {
      final f0 = start + a < flags.length ? flags[start + a] : 0;
      final f1 = i < line.length
          ? (start + i < flags.length ? flags[start + i] : 0)
          : -1;
      if (f1 != f0) {
        spans.add(
          Span(
            line.substring(a, i),
            bold: f0 & (Fx.bold | Fx.strong) != 0,
            italic: f0 & Fx.italic != 0,
            underline: f0 & Fx.underline != 0,
            code: f0 & Fx.code != 0,
            at: f0 & Fx.at != 0,
            link: f0 & Fx.link != 0,
          ),
        );
        a = i;
      }
    }
    return spans;
  }

  /// Charge des blocs reçus dans l'éditeur (paragraphes séparés par une ligne vide).
  void loadBlocks(List<Block> blocks) {
    final sb = StringBuffer();
    final f = <int>[];
    var first = true;
    void walk(List<Block> bs) {
      for (final b in bs) {
        switch (b) {
          case Para():
            if (!first) {
              sb.write('\n\n');
              f.addAll([0, Fx.gap]);
            }
            first = false;
            var spans = b.spans;
            for (var i = 0; i < spans.length; i++) {
              var tx = spans[i].text;
              if (i == spans.length - 1 && tx.endsWith('\n')) {
                tx = tx.substring(0, tx.length - 1);
              }
              final s = spans[i];
              final bits =
                  (s.bold ? Fx.strong : 0) |
                  (s.italic ? Fx.italic : 0) |
                  (s.underline ? Fx.underline : 0) |
                  (s.code ? Fx.code : 0) |
                  (s.at ? Fx.at : 0) |
                  (s.link ? Fx.link : 0);
              sb.write(tx);
              f.addAll(List<int>.filled(tx.length, bits));
            }
          case Quote():
            walk(b.children);
        }
      }
    }

    walk(blocks);
    setContent(sb.toString(), f);
  }
}
