import 'dart:async';

import 'package:flutter/material.dart';

import '../data/rule_models.dart';
import '../engine/rule_admin.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'rule_editor.dart';
import 'widgets.dart';

/// Règles « condition → action » : liste réordonnable, activation, édition, suppression.
class RulesSettings extends StatefulWidget {
  final RuleAdmin admin;
  final Map<String, String> accounts;
  final Stream<void> changes;
  const RulesSettings({
    super.key,
    required this.admin,
    required this.accounts,
    required this.changes,
  });

  @override
  State<RulesSettings> createState() => _RulesSettingsState();
}

class _RulesSettingsState extends State<RulesSettings> {
  List<RuleView> views = [];
  RuleCatalog catalog = const RuleCatalog([], []);
  Map<String, List<RuleChoice>> mailboxes = {};
  Rule? editing;
  String? error;
  StreamSubscription<void>? _sub;

  @override
  void initState() {
    super.initState();
    _sub = widget.changes.listen((_) => _load());
    _load();
    _loadMailboxes();
  }

  /// Boîtes de chaque compte, pour les listes de l'éditeur ; un compte illisible n'en propose aucune.
  Future<void> _loadMailboxes() async {
    final found = <String, List<RuleChoice>>{};
    for (final id in widget.accounts.keys) {
      try {
        found[id] = await widget.admin.mailboxes(id);
      } catch (_) {
        found[id] = const [];
      }
    }
    if (mounted) setState(() => mailboxes = found);
  }

  @override
  void dispose() {
    _sub?.cancel();
    super.dispose();
  }

  Future<void> _load() async {
    try {
      final list = await widget.admin.list();
      final cat = await widget.admin.catalog();
      if (!mounted) return;
      setState(() {
        views = list;
        catalog = cat;
        error = null;
      });
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    }
  }

  Future<void> _run(Future<void> Function() op) async {
    try {
      await op();
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    }
    await _load();
  }

  Future<void> _save(Rule rule) async {
    if (rule.id.isEmpty) {
      await widget.admin.create(rule);
    } else {
      await widget.admin.update(rule);
    }
    if (mounted) setState(() => editing = null);
    await _load();
  }

  Future<void> _reorder(int from, int to) async {
    final next = [...views];
    final moved = next.removeAt(from);
    next.insert(to, moved);
    setState(() => views = next);
    await _run(() => widget.admin.reorder([for (final v in next) v.rule.id]));
  }

  Future<void> _confirmDelete(Rule rule) async {
    final ok = await showDialog<bool>(
      context: context,
      builder: (c) => AlertDialog(
        title: Text(t('Supprimer la règle « {0} » ?', [rule.name])),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(c, false),
            child: Text(t('Annuler')),
          ),
          TextButton(
            key: const ValueKey('confirm-delete-rule'),
            onPressed: () => Navigator.pop(c, true),
            child: Text(t('Supprimer')),
          ),
        ],
      ),
    );
    if (ok == true) await _run(() => widget.admin.delete(rule.id));
  }

  Widget _row(int i) {
    final v = views[i];
    final p = context.pal;
    return ListTile(
      key: ValueKey('rule-row-${v.rule.id}'),
      leading: ReorderableDragStartListener(
        index: i,
        child: const Icon(Icons.drag_handle),
      ),
      title: Text(v.rule.name),
      subtitle: v.status.valid
          ? null
          : Text(
              key: ValueKey('rule-invalid-${v.rule.id}'),
              '${t('Invalide')} : ${v.status.reason}',
              style: Fonts.sans(size: 12.5, color: p.dim),
            ),
      onTap: () => setState(() => editing = v.rule),
      trailing: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Switch(
            key: ValueKey('rule-enabled-${v.rule.id}'),
            value: v.rule.enabled,
            onChanged: (on) =>
                _run(() => widget.admin.setEnabled(v.rule.id, on)),
          ),
          IconButton(
            key: ValueKey('rule-delete-${v.rule.id}'),
            tooltip: t('Supprimer'),
            icon: const Icon(Icons.delete_outline),
            onPressed: () => _confirmDelete(v.rule),
          ),
        ],
      ),
    );
  }

  Widget _list() {
    final p = context.pal;
    return Padding(
      padding: const EdgeInsets.all(20),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text(
            t('Règles'),
            style: Fonts.sans(size: 15, w: FontWeight.w500, color: p.text),
          ),
          const SizedBox(height: 2),
          Text(
            t(
              'Appliquées dans l\'ordre aux nouveaux messages : si les conditions sont remplies, les actions s\'exécutent.',
            ),
            style: Fonts.sans(size: 13.5, color: p.dim),
          ),
          if (error != null)
            Text(error!, style: Fonts.sans(size: 13, color: p.dim)),
          const SizedBox(height: 10),
          if (views.isEmpty)
            Text(
              t('Aucune règle.'),
              style: Fonts.sans(size: 13.5, color: p.dim),
            ),
          ReorderableListView.builder(
            shrinkWrap: true,
            buildDefaultDragHandles: false,
            physics: const NeverScrollableScrollPhysics(),
            itemCount: views.length,
            onReorderItem: _reorder,
            itemBuilder: (_, i) => _row(i),
          ),
          const SizedBox(height: 10),
          Align(
            alignment: Alignment.centerLeft,
            child: Btn(
              key: const ValueKey('add-rule'),
              label: t('Ajouter une règle'),
              onTap: () => setState(
                () => editing = Rule(id: '', name: t('Nouvelle règle')),
              ),
            ),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final rule = editing;
    if (rule == null) return _list();
    return RuleEditor(
      key: ValueKey('editor-${rule.id}'),
      rule: rule,
      catalog: catalog,
      accounts: widget.accounts,
      mailboxes: mailboxes,
      onSave: _save,
      onPreview: widget.admin.preview,
      onCancel: () => setState(() => editing = null),
    );
  }
}
