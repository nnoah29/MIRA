import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';

/// Tracés des icônes (symboles `#i-*` du HTML d'origine), viewBox 24×24.
const _paths = <String, String>{
  'inbox': '<path d="M3 13l3-8h12l3 8v6H3z"/><path d="M3 13h5l1 3h6l1-3h5"/>',
  'star': '<path d="M12 3l2.7 5.6 6.1.8-4.5 4.2 1.1 6-5.4-3-5.4 3 1.1-6L3.2 9.4l6.1-.8z"/>',
  'send': '<path d="M21 3L10 14"/><path d="M21 3l-7 18-4-7-7-4z"/>',
  'file': '<path d="M6 3h8l5 5v13H6z"/><path d="M14 3v5h5"/>',
  'warn': '<path d="M12 3l10 18H2z"/><path d="M12 10v5M12 18v.5"/>',
  'trash': '<path d="M4 7h16M9 7V4h6v3M6 7l1 14h10l1-14"/>',
  'archive': '<rect x="3" y="4" width="18" height="5"/><path d="M5 9v11h14V9M10 13h4"/>',
  'search': '<circle cx="11" cy="11" r="7"/><path d="M20 20l-4-4"/>',
  'pen': '<path d="M4 20l1-5L16 4l4 4L9 19z"/>',
  'clip': '<path d="M20 11l-8.5 8.5a5 5 0 01-7-7L13 4a3.5 3.5 0 015 5l-8.5 8.5a2 2 0 01-3-3L14 7"/>',
  'reply': '<path d="M10 8L4 13l6 5"/><path d="M4 13h10a6 6 0 016 6"/>',
  'dots': '<circle cx="5" cy="12" r="1"/><circle cx="12" cy="12" r="1"/><circle cx="19" cy="12" r="1"/>',
  'tag': '<path d="M3 12V3h9l9 9-9 9z"/><circle cx="8" cy="8" r="1"/>',
  'diamond': '<path d="M12 3l9 9-9 9-9-9z"/>',
  'check': '<path d="M5 12l5 5 9-10"/>',
  'x': '<path d="M6 6l12 12M18 6L6 18"/>',
  'chev': '<path d="M6 9l6 6 6-6"/>',
  'filter': '<path d="M4 7h16M7 12h10M10 17h4"/>',
  'clock': '<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>',
  'link': '<path d="M10 14a4 4 0 005.7 0l3-3a4 4 0 00-5.7-5.7l-1 1"/><path d="M14 10a4 4 0 00-5.7 0l-3 3a4 4 0 005.7 5.7l1-1"/>',
  'list': '<path d="M9 6h11M9 12h11M9 18h11M4 6h.5M4 12h.5M4 18h.5"/>',
  'alignl': '<path d="M4 6h16M4 10h10M4 14h16M4 18h10"/>',
  'alignc': '<path d="M4 6h16M7 10h10M4 14h16M7 18h10"/>',
  'gear': '<circle cx="12" cy="12" r="3"/><path d="M12 2v3M12 19v3M2 12h3M19 12h3M5 5l2 2M17 17l2 2M19 5l-2 2M7 17l-2 2"/>',
  'help': '<circle cx="12" cy="12" r="9"/><path d="M9.5 9.5a2.5 2.5 0 015 .5c0 1.5-2.5 2-2.5 3.5M12 17v.5"/>',
  'minus': '<path d="M5 12h14"/>',
  'download': '<path d="M12 4v11M7 11l5 5 5-5M5 20h14"/>',
  'plus': '<path d="M12 5v14M5 12h14"/>',
  'bot': '<rect x="4" y="8" width="16" height="12"/><path d="M12 4v4M9 13v2M15 13v2"/>',
  'spark': '<path d="M12 3l2 6 6 3-6 3-2 6-2-6-6-3 6-3z"/>',
  'img':
      '<path d="M21 8l-9-5-9 5v8l9 5 9-5z"/><path d="M3 8l9 5 9-5M12 13v8"/>',
  'fwd': '<path d="M14 8l6 5-6 5"/><path d="M20 13H10a6 6 0 00-6 6"/>',
  'arrowl': '<path d="M19 12H5M11 6l-6 6 6 6"/>',
};

final _cache = <String, String>{};

String _svg(String name, double sw, bool filled) => _cache.putIfAbsent(
  '$name|$sw|$filled',
  () {
    return '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" '
        'fill="${filled ? '#000' : 'none'}" stroke="#000" stroke-width="$sw" '
        'stroke-linecap="round" stroke-linejoin="round">${_paths[name]}</svg>';
  },
);

/// Icône trait (16 px par défaut, trait 1,6), teintée par [color].
class Ico extends StatelessWidget {
  final String name;
  final double size;
  final Color? color;
  final double stroke;
  final bool filled;
  const Ico(
    this.name, {
    super.key,
    this.size = 16,
    this.color,
    this.stroke = 1.6,
    this.filled = false,
  });

  @override
  Widget build(BuildContext context) {
    final c = color ?? DefaultTextStyle.of(context).style.color ?? Colors.white;
    return SvgPicture.string(
      _svg(name, stroke, filled),
      width: size,
      height: size,
      colorFilter: ColorFilter.mode(c, BlendMode.srcIn),
    );
  }
}
