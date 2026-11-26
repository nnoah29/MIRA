import 'package:flutter/material.dart';

import '../state/gif_playback.dart';

/// Image d'un message qui peut être animée (GIF) : figée sur sa première image au repos,
/// animée sous la souris ou en permanence selon le réglage. Figée, elle ne demande plus aucune image à l'écran.
class GifImage extends StatefulWidget {
  final ImageProvider image;
  final GifPlayback playback;
  final double? width;
  final double? height;
  final BoxFit? fit;
  final ImageErrorWidgetBuilder? errorBuilder;
  const GifImage({
    super.key,
    required this.image,
    required this.playback,
    this.width,
    this.height,
    this.fit,
    this.errorBuilder,
  });

  @override
  State<GifImage> createState() => _GifImageState();
}

class _GifImageState extends State<GifImage> {
  ImageStream? _stream;
  ImageStreamListener? _listener;
  ImageInfo? _still;
  (Object, StackTrace?)? _error;
  bool _hovered = false;

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (_stream == null) _capture();
  }

  @override
  void didUpdateWidget(GifImage old) {
    super.didUpdateWidget(old);
    if (old.image != widget.image) _capture();
  }

  @override
  void dispose() {
    _release();
    super.dispose();
  }

  /// Récupère la première image puis cesse d'écouter : sans auditeur, le décodeur n'avance plus.
  void _capture() {
    _release();
    _still?.dispose();
    _still = null;
    _error = null;
    final stream = widget.image.resolve(createLocalImageConfiguration(context));
    final listener = ImageStreamListener(_onFirstFrame, onError: _onError);
    _stream = stream;
    _listener = listener;
    stream.addListener(listener);
  }

  void _onFirstFrame(ImageInfo info, bool synchronous) {
    _still?.dispose();
    _still = info.clone();
    _release();
    if (mounted) setState(() {});
  }

  void _onError(Object error, StackTrace? trace) {
    _release();
    if (mounted) setState(() => _error = (error, trace));
  }

  void _release() {
    final listener = _listener;
    if (listener != null) _stream?.removeListener(listener);
    _listener = null;
  }

  bool get _playing =>
      widget.playback == GifPlayback.always ||
      (widget.playback == GifPlayback.hover && _hovered);

  Widget _frame(BuildContext context) {
    final error = _error;
    if (error != null) {
      return widget.errorBuilder?.call(context, error.$1, error.$2) ??
          const SizedBox.shrink();
    }
    if (_playing) {
      return Image(
        image: widget.image,
        width: widget.width,
        height: widget.height,
        fit: widget.fit,
        errorBuilder: widget.errorBuilder,
      );
    }
    final still = _still;
    if (still == null) {
      return SizedBox(width: widget.width, height: widget.height);
    }
    return RawImage(
      image: still.image,
      scale: still.scale,
      width: widget.width,
      height: widget.height,
      fit: widget.fit,
    );
  }

  @override
  Widget build(BuildContext context) {
    final frame = _frame(context);
    if (widget.playback != GifPlayback.hover) return frame;
    return MouseRegion(
      onEnter: (_) => setState(() => _hovered = true),
      onExit: (_) => setState(() => _hovered = false),
      child: frame,
    );
  }
}

/// Cette image est-elle un GIF (donc peut-être animée) ? Décidé sur les octets d'une image incorporée, ou sur l'extension de l'adresse d'une image distante.
bool isGifSource(ImageProvider image) => switch (image) {
  MemoryImage(:final bytes) =>
    bytes.length >= 4 &&
        bytes[0] == 0x47 &&
        bytes[1] == 0x49 &&
        bytes[2] == 0x46 &&
        bytes[3] == 0x38,
  NetworkImage(:final url) =>
    (Uri.tryParse(url)?.path.toLowerCase() ?? '').endsWith('.gif'),
  _ => false,
};
