import 'package:flutter/material.dart';

import '../engine/remote/resident.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'account_form_widgets.dart';

/// Interrupteur « Garder MIRA actif en arrière-plan » des réglages ; absent si l'unité n'est pas installée.
class ResidentRow extends StatefulWidget {
  const ResidentRow({super.key, this.controller});

  /// Contrôleur à utiliser (le vrai par défaut).
  final ResidentController? controller;

  @override
  State<ResidentRow> createState() => _ResidentRowState();
}

class _ResidentRowState extends State<ResidentRow> {
  late final ResidentController _controller =
      widget.controller ?? ResidentController();
  bool _available = false;
  bool _enabled = false;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final available = await _controller.isAvailable();
    final enabled = available && await _controller.isEnabled();
    if (mounted) {
      setState(() {
        _available = available;
        _enabled = enabled;
      });
    }
  }

  Future<void> _toggle(bool on) async {
    try {
      on ? await _controller.enable() : await _controller.disable();
      if (mounted) setState(() => _enabled = on);
    } catch (_) {
      await _load();
    }
  }

  @override
  Widget build(BuildContext context) {
    if (!_available) return const SizedBox.shrink();
    final p = context.pal;
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 16),
      child: Wrap(
        alignment: WrapAlignment.spaceBetween,
        crossAxisAlignment: WrapCrossAlignment.center,
        runSpacing: 12,
        spacing: 24,
        children: [
          ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 480),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  t('Garder MIRA actif en arrière-plan'),
                  style: Fonts.sans(
                    size: 15,
                    w: FontWeight.w500,
                    color: p.text,
                  ),
                ),
                Text(
                  t(
                    'Les notifications et la synchronisation continuent fenêtre fermée.',
                  ),
                  style: Fonts.sans(size: 13.5, color: p.dim),
                ),
              ],
            ),
          ),
          Segments<String>(
            keyPrefix: 'resident',
            options: [('1', t('Activé')), ('0', t('Désactivé'))],
            value: _enabled ? '1' : '0',
            onChanged: (v) => _toggle(v == '1'),
          ),
        ],
      ),
    );
  }
}
