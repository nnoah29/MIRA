import 'dart:async';

import 'package:flutter/material.dart';

import '../engine/account_admin.dart';
import '../engine/engine_client.dart';
import '../i18n/i18n.dart';
import '../theme/app_theme.dart';
import 'account_form_widgets.dart';
import 'widgets.dart';

final _addr = RegExp(r'^[^\s@<>,;]+@[^\s@<>,;]+\.[^\s@<>,;]+$');

/// Identités d'envoi d'un compte : liste, modification du nom et de la signature, identité par défaut, ajout.
class IdentitiesBox extends StatefulWidget {
  final AccountDetail account;
  final AccountAdmin admin;
  const IdentitiesBox({super.key, required this.account, required this.admin});

  @override
  State<IdentitiesBox> createState() => _IdentitiesBoxState();
}

class _IdentitiesBoxState extends State<IdentitiesBox> {
  List<IdentityDetail>? items;
  String? error;
  final newAddress = TextEditingController();
  final newName = TextEditingController();
  final newSignature = TextEditingController();
  bool newDefault = false, busy = false;

  @override
  void initState() {
    super.initState();
    unawaited(load());
  }

  @override
  void dispose() {
    newAddress.dispose();
    newName.dispose();
    newSignature.dispose();
    super.dispose();
  }

  Future<void> load() async {
    try {
      final list = await widget.admin.identities(widget.account.id);
      if (mounted) setState(() => items = list);
    } catch (e) {
      if (mounted) setState(() => error = _msg(e));
    }
  }

  String _msg(Object e) =>
      e is EngineException ? e.message : t('Erreur inattendue : {0}', ['$e']);

  Future<void> _run(Future<void> Function() job) async {
    setState(() {
      busy = true;
      error = null;
    });
    try {
      await job();
      await load();
    } catch (e) {
      if (mounted) setState(() => error = _msg(e));
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  Future<void> _add() => _run(() async {
    final address = newAddress.text.trim();
    if (!_addr.hasMatch(address)) {
      throw EngineException(
        'invalid_input.address',
        t('Adresse e-mail invalide.'),
      );
    }
    final name = newName.text.trim();
    await widget.admin.addIdentity(
      accountId: widget.account.id,
      address: address,
      displayName: name.isEmpty ? address : name,
      signature: newSignature.text.isEmpty ? null : newSignature.text,
      isDefault: newDefault,
    );
    newAddress.clear();
    newName.clear();
    newSignature.clear();
    newDefault = false;
  });

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final list = items;
    return Padding(
      padding: const EdgeInsets.only(bottom: 18),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Mono(
            t('IDENTITÉS D’ENVOI'),
            size: 10.5,
            ls: .12,
            color: p.muted,
            height: null,
          ),
          const SizedBox(height: 8),
          if (list == null)
            const SizedBox(
              height: 24,
              child: Center(child: LinearProgressIndicator()),
            )
          else ...[
            for (final i in list)
              _IdentityRow(
                key: ValueKey('identity-${i.id}'),
                identity: i,
                busy: busy,
                onSave: (name, sig) => _run(
                  () => widget.admin.updateIdentity(
                    i.id,
                    displayName: name,
                    signature: sig,
                  ),
                ),
                onDefault: () =>
                    _run(() => widget.admin.setDefaultIdentity(i.id)),
              ),
            if (list.isEmpty)
              Text(
                t('Aucune identité.'),
                style: Fonts.sans(size: 13.5, color: p.dim),
              ),
          ],
          const SizedBox(height: 12),
          Container(
            padding: const EdgeInsets.all(14),
            decoration: BoxDecoration(
              color: p.bg,
              border: Border.all(color: p.line),
            ),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  t('Ajouter une identité (alias, autre nom)'),
                  style: Fonts.sans(
                    size: 14,
                    w: FontWeight.w500,
                    color: p.text,
                  ),
                ),
                const SizedBox(height: 10),
                Wrap(
                  spacing: 12,
                  runSpacing: 10,
                  children: [
                    FormField2(
                      fieldKey: ValueKey('new-id-address-${widget.account.id}'),
                      label: t('ADRESSE'),
                      controller: newAddress,
                      width: 280,
                      hint: 'alias@exemple.com',
                    ),
                    FormField2(
                      fieldKey: ValueKey('new-id-name-${widget.account.id}'),
                      label: t('NOM AFFICHÉ'),
                      controller: newName,
                      width: 240,
                    ),
                  ],
                ),
                const SizedBox(height: 10),
                FormField2(
                  fieldKey: ValueKey('new-id-sig-${widget.account.id}'),
                  label: t('SIGNATURE'),
                  controller: newSignature,
                  width: 540,
                ),
                const SizedBox(height: 10),
                Wrap(
                  spacing: 14,
                  crossAxisAlignment: WrapCrossAlignment.center,
                  children: [
                    Segments<bool>(
                      keyPrefix: 'new-id-default-${widget.account.id}',
                      options: [
                        (false, t('Secondaire')),
                        (true, t('Par défaut')),
                      ],
                      value: newDefault,
                      onChanged: (v) => setState(() => newDefault = v),
                    ),
                    Btn(
                      key: ValueKey('add-identity-${widget.account.id}'),
                      icon: 'plus',
                      label: t('Ajouter'),
                      onTap: busy ? null : _add,
                    ),
                  ],
                ),
              ],
            ),
          ),
          if (error != null)
            Padding(
              padding: const EdgeInsets.only(top: 10),
              child: Text(
                error!,
                key: ValueKey('identity-error-${widget.account.id}'),
                style: Fonts.sans(size: 14, color: p.accent),
              ),
            ),
        ],
      ),
    );
  }
}

class _IdentityRow extends StatefulWidget {
  final IdentityDetail identity;
  final bool busy;
  final void Function(String name, String? signature) onSave;
  final VoidCallback onDefault;
  const _IdentityRow({
    super.key,
    required this.identity,
    required this.busy,
    required this.onSave,
    required this.onDefault,
  });

  @override
  State<_IdentityRow> createState() => _IdentityRowState();
}

class _IdentityRowState extends State<_IdentityRow> {
  late final name = TextEditingController(text: widget.identity.displayName);
  late final sig = TextEditingController(text: widget.identity.signature ?? '');

  @override
  void dispose() {
    name.dispose();
    sig.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final p = context.pal;
    final i = widget.identity;
    return Container(
      margin: const EdgeInsets.only(bottom: 10),
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        border: Border.all(color: i.isDefault ? p.accent : p.line),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            children: [
              Expanded(
                child: Text(
                  i.address,
                  style: Fonts.sans(
                    size: 15,
                    w: FontWeight.w500,
                    color: p.text,
                  ),
                ),
              ),
              if (i.isDefault)
                Mono(
                  t('PAR DÉFAUT'),
                  size: 10.5,
                  ls: .12,
                  color: p.accent,
                  height: null,
                )
              else
                Btn(
                  key: ValueKey('default-${i.id}'),
                  label: t('Définir par défaut'),
                  onTap: widget.busy ? null : widget.onDefault,
                ),
            ],
          ),
          const SizedBox(height: 10),
          Wrap(
            spacing: 12,
            runSpacing: 10,
            crossAxisAlignment: WrapCrossAlignment.end,
            children: [
              FormField2(
                fieldKey: ValueKey('id-name-${i.id}'),
                label: t('NOM AFFICHÉ'),
                controller: name,
                width: 260,
              ),
              FormField2(
                fieldKey: ValueKey('id-sig-${i.id}'),
                label: t('SIGNATURE'),
                controller: sig,
                width: 340,
              ),
              Btn(
                key: ValueKey('save-identity-${i.id}'),
                label: t('Enregistrer'),
                onTap: widget.busy
                    ? null
                    : () => widget.onSave(
                        name.text.trim().isEmpty
                            ? i.displayName
                            : name.text.trim(),
                        sig.text,
                      ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
