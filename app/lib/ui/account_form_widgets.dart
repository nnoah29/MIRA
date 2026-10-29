import 'package:flutter/material.dart';

import '../data/format.dart';
import '../engine/account_setup.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'widgets.dart';

/// Titre de section du panneau d'ajout de compte.
class FormSection extends StatelessWidget {
  final String title;
  final String? hint;
  final Widget child;
  const FormSection({
    super.key,
    required this.title,
    this.hint,
    required this.child,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Container(
      margin: const EdgeInsets.only(top: 20),
      padding: const EdgeInsets.fromLTRB(20, 16, 20, 18),
      decoration: BoxDecoration(
        color: p.surface,
        border: Border.all(color: p.line),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            title,
            style: Fonts.sans(size: 15, w: FontWeight.w500, color: p.text),
          ),
          if (hint != null)
            Text(hint!, style: Fonts.sans(size: 13.5, color: p.dim)),
          const SizedBox(height: 14),
          child,
        ],
      ),
    );
  }
}

/// Carte-bouton d'un fournisseur (monogramme coloré + nom).
class ProviderCard extends StatelessWidget {
  final String name;
  final String? subtitle;
  final bool selected;
  final VoidCallback onTap;
  const ProviderCard({
    super.key,
    required this.name,
    this.subtitle,
    required this.selected,
    required this.onTap,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Hov(
      onTap: onTap,
      builder: (_, h) => AnimatedContainer(
        duration: const Duration(milliseconds: 130),
        width: 188,
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: selected ? p.accentSoft : (h ? p.surface2 : p.bg),
          border: Border.all(color: selected ? p.accent : p.line),
        ),
        child: Row(
          children: [
            Container(
              width: 34,
              height: 34,
              alignment: Alignment.center,
              color: Color(colorOf(name)),
              child: Text(
                name.characters.first.toUpperCase(),
                style: Fonts.sans(
                  size: 16,
                  w: FontWeight.w700,
                  color: const Color(0xFF1b1b1b),
                ),
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    name,
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: Fonts.sans(
                      size: 15,
                      w: FontWeight.w500,
                      color: selected ? p.accent : p.text,
                    ),
                  ),
                  if (subtitle != null)
                    Text(
                      subtitle!,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: Fonts.sans(size: 12, color: p.dim),
                    ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// Champ de texte au style des réglages.
class FormField2 extends StatelessWidget {
  final String label;
  final TextEditingController controller;
  final String? hint;
  final bool secret;
  final TextInputType? keyboard;
  final ValueChanged<String>? onChanged;
  final Widget? trailing;
  final double? width;
  final Key? fieldKey;
  final int maxLines;
  final ValueChanged<String>? onSubmitted;
  final bool autofocus;
  const FormField2({
    super.key,
    required this.label,
    required this.controller,
    this.hint,
    this.secret = false,
    this.keyboard,
    this.onChanged,
    this.trailing,
    this.width,
    this.fieldKey,
    this.maxLines = 1,
    this.onSubmitted,
    this.autofocus = false,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final mobile = MediaQuery.sizeOf(context).width <= 760;
    return SizedBox(
      width: mobile && width != null ? double.infinity : width,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          if (label.isNotEmpty) ...[
            Mono(label, size: 10.5, ls: .12, color: p.muted, height: null),
            const SizedBox(height: 6),
          ],
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            decoration: BoxDecoration(
              color: p.bg,
              border: Border.all(color: p.line),
            ),
            child: Row(
              children: [
                Expanded(
                  child: TextField(
                    key: fieldKey,
                    controller: controller,
                    obscureText: secret,
                    enableSuggestions: !secret,
                    autocorrect: false,
                    keyboardType: maxLines > 1
                        ? TextInputType.multiline
                        : keyboard,
                    minLines: maxLines,
                    maxLines: maxLines,
                    onChanged: onChanged,
                    onSubmitted: onSubmitted,
                    autofocus: autofocus,
                    cursorColor: p.accent,
                    cursorWidth: 1,
                    style: Fonts.sans(size: 15, color: p.text),
                    decoration: InputDecoration(
                      isDense: true,
                      border: InputBorder.none,
                      contentPadding: EdgeInsets.zero,
                      hintText: hint,
                      hintStyle: Fonts.sans(size: 15, color: p.dim),
                    ),
                  ),
                ),
                ?trailing,
              ],
            ),
          ),
        ],
      ),
    );
  }
}

/// Choix exclusif présenté en segments.
class Segments<T> extends StatelessWidget {
  final List<(T, String)> options;
  final T value;
  final ValueChanged<T> onChanged;
  final String keyPrefix;
  const Segments({
    super.key,
    required this.options,
    required this.value,
    required this.onChanged,
    required this.keyPrefix,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final narrow = MediaQuery.sizeOf(context).width <= 760;
    return LayoutBuilder(
      builder: (_, box) {
        final stacked = narrow && box.hasBoundedWidth;
        final cells = [
          for (final (i, o) in options.indexed) _cell(p, i, o, stacked),
        ];
        return Container(
          decoration: BoxDecoration(border: Border.all(color: p.line)),
          child: stacked
              ? Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: cells,
                )
              : Row(mainAxisSize: MainAxisSize.min, children: cells),
        );
      },
    );
  }

  /// Une cellule du choix, séparée de la précédente par un filet.
  Widget _cell(Pal p, int i, (T, String) o, bool mobile) => Hov(
    key: ValueKey('$keyPrefix-${o.$1}'),
    onTap: () => onChanged(o.$1),
    builder: (_, h) => Container(
      padding: const EdgeInsets.symmetric(horizontal: 14, vertical: 9),
      decoration: BoxDecoration(
        color: value == o.$1 ? p.accentSoft : null,
        border: i == 0
            ? null
            : mobile
            ? Border(top: BorderSide(color: p.line))
            : Border(left: BorderSide(color: p.line)),
      ),
      child: Mono(
        o.$2,
        ls: .12,
        color: value == o.$1 ? p.accent : (h ? p.text : p.muted),
        height: null,
      ),
    ),
  );
}

/// Libellé lisible d'une méthode d'authentification du moteur.
String methodLabel(String m) => switch (m) {
  'oauth2' => t('OAuth (navigateur)'),
  'app_password' => t('Mot de passe d’application'),
  _ => t('Mot de passe'),
};

/// Libellé d'un mode de chiffrement.
String securityLabel(String s) => switch (s) {
  'tls' => 'TLS',
  'start_tls' => 'STARTTLS',
  _ => t('Aucun'),
};

/// Saisie d'un serveur (hôte, port, chiffrement).
class ServerFields extends StatelessWidget {
  final String title;
  final String keyPrefix;
  final TextEditingController host, port;
  final String security;
  final ValueChanged<String> onSecurity;
  final VoidCallback onEdited;
  const ServerFields({
    super.key,
    required this.title,
    required this.keyPrefix,
    required this.host,
    required this.port,
    required this.security,
    required this.onSecurity,
    required this.onEdited,
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Mono(title, size: 11, ls: .14, color: p.accent, height: null),
        const SizedBox(height: 10),
        Wrap(
          spacing: 14,
          runSpacing: 12,
          crossAxisAlignment: WrapCrossAlignment.end,
          children: [
            FormField2(
              fieldKey: ValueKey('$keyPrefix-host'),
              label: t('SERVEUR'),
              controller: host,
              width: 280,
              hint: keyPrefix == 'imap'
                  ? 'imap.exemple.com'
                  : 'smtp.exemple.com',
              onChanged: (_) => onEdited(),
            ),
            FormField2(
              fieldKey: ValueKey('$keyPrefix-port'),
              label: t('PORT'),
              controller: port,
              width: 90,
              keyboard: TextInputType.number,
              onChanged: (_) => onEdited(),
            ),
            Segments<String>(
              keyPrefix: '$keyPrefix-sec',
              options: [for (final s in securities) (s, securityLabel(s))],
              value: security,
              onChanged: onSecurity,
            ),
          ],
        ),
      ],
    );
  }
}

/// Deux blocs côte à côte sur grand écran, empilés sur mobile.
class SideBySide extends StatelessWidget {
  final double leftWidth;
  final Widget left, right;
  const SideBySide({
    super.key,
    required this.leftWidth,
    required this.left,
    required this.right,
  });

  @override
  Widget build(BuildContext context) {
    if (MediaQuery.sizeOf(context).width <= 760) {
      return Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [left, const SizedBox(height: 10), right],
      );
    }
    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(width: leftWidth, child: left),
        const SizedBox(width: 12),
        Expanded(child: right),
      ],
    );
  }
}
