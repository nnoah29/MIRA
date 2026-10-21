import 'package:flutter/scheduler.dart';
import 'package:flutter/widgets.dart';

/// Les vues natives (vue web GTK sous Linux) sont posées par-dessus Flutter : ce que Flutter superpose passerait dessous.
/// Les popovers ouverts publient ici leur rectangle à l'écran, les modales hors navigateur leur nombre.
class NativeCover {
  static final covers = ValueNotifier<Map<Object, Rect>>(const {});
  static final modals = ValueNotifier<int>(0);

  static void _set(Object id, Rect? rect) => _later(() {
    final next = Map.of(covers.value);
    rect == null ? next.remove(id) : next[id] = rect;
    covers.value = next;
  });

  /// Un changement pendant la construction d'une image attend la fin de celle-ci (les auditeurs reconstruisent).
  static void _later(VoidCallback apply) {
    final binding = SchedulerBinding.instance;
    if (binding.schedulerPhase == SchedulerPhase.persistentCallbacks) {
      binding.addPostFrameCallback((_) => apply());
      binding.ensureVisualUpdate();
    } else {
      apply();
    }
  }
}

/// Tant qu'il est affiché, son enfant passe devant les vues natives : celles qu'il chevauche se replient,
/// ou, pour une modale (`modal`), se masquent le temps qu'elle est ouverte.
class CoversNativeViews extends StatefulWidget {
  final Widget child;
  final bool modal;
  const CoversNativeViews({super.key, required this.child, this.modal = false});

  @override
  State<CoversNativeViews> createState() => _CoversNativeViewsState();
}

class _CoversNativeViewsState extends State<CoversNativeViews> {
  @override
  void initState() {
    super.initState();
    if (widget.modal) {
      NativeCover._later(() => NativeCover.modals.value++);
    } else {
      WidgetsBinding.instance.addPostFrameCallback((_) => _publish());
    }
  }

  /// Le rectangle n'est connu qu'une fois l'enfant mis en page.
  void _publish() {
    final box = context.findRenderObject();
    if (!mounted || box is! RenderBox || !box.hasSize) return;
    NativeCover._set(this, box.localToGlobal(Offset.zero) & box.size);
  }

  @override
  void dispose() {
    if (widget.modal) {
      NativeCover._later(() => NativeCover.modals.value--);
    } else {
      NativeCover._set(this, null);
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => widget.child;
}

/// Construit son enfant selon qu'une modale (boîte de dialogue, aide) couvre la page, et signale (`onOverlapped`) chaque superposition qui s'ouvre par-dessus lui.
/// Un widget hors écran (mail gardé monté mais pas affiché, `TickerMode` coupé) n'est pas concerné.
class NativeCoverBuilder extends StatefulWidget {
  final Widget Function(BuildContext context, bool covered) builder;
  final VoidCallback? onOverlapped;
  const NativeCoverBuilder({
    super.key,
    required this.builder,
    this.onOverlapped,
  });

  @override
  State<NativeCoverBuilder> createState() => _NativeCoverBuilderState();
}

class _NativeCoverBuilderState extends State<NativeCoverBuilder> {
  final _seen = <Object>{};
  bool _onScreen = true;

  @override
  void initState() {
    super.initState();
    _seen.addAll(NativeCover.covers.value.keys);
    NativeCover.covers.addListener(_update);
  }

  @override
  void dispose() {
    NativeCover.covers.removeListener(_update);
    super.dispose();
  }

  /// Seules les superpositions nouvellement ouvertes comptent : leur chevauchement est jugé une fois, à l'ouverture.
  void _update() {
    final covers = NativeCover.covers.value;
    final opened = [
      for (final MapEntry(:key, :value) in covers.entries)
        if (!_seen.contains(key)) value,
    ];
    _seen
      ..retainAll(covers.keys)
      ..addAll(covers.keys);
    final box = context.findRenderObject();
    if (!_onScreen || box is! RenderBox || !box.hasSize || !box.attached) {
      return;
    }
    final area = box.localToGlobal(Offset.zero) & box.size;
    if (opened.any(area.overlaps)) widget.onOverlapped?.call();
  }

  @override
  Widget build(BuildContext context) {
    _onScreen = TickerMode.valuesOf(context).enabled;
    final current = ModalRoute.isCurrentOf(context) ?? true;
    return ValueListenableBuilder(
      valueListenable: NativeCover.modals,
      builder: (context, modals, _) =>
          widget.builder(context, !current || modals > 0),
    );
  }
}
