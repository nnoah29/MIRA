/// Gel des animations d'un courriel quand la fenêtre n'est plus au premier plan.
/// Définit `window.__miraPause(true|false)` dans la page :
/// les images remplacées par un cliché fixe, les animations CSS et SVG mises en pause, puis tout est rendu au retour du focus.
/// Un GIF animé redémarre alors depuis sa première image.
const webPagePauseScript = '''
(function() {
  if (window.__miraPause) return;
  var maxPixels = 4000000;
  var swapped = [];
  var paused = [];
  var layout = ['display', 'verticalAlign', 'float', 'marginTop', 'marginRight', 'marginBottom', 'marginLeft', 'borderRadius'];
  function snapshot(img) {
    if (!img.complete || !img.naturalWidth || img.naturalWidth * img.naturalHeight > maxPixels) return;
    var box = img.getBoundingClientRect();
    if (!box.width || !box.height) return;
    var canvas = document.createElement('canvas');
    canvas.width = img.naturalWidth;
    canvas.height = img.naturalHeight;
    try { canvas.getContext('2d').drawImage(img, 0, 0); } catch (_) { return; }
    var look = getComputedStyle(img);
    canvas.className = img.className;
    canvas.style.cssText = img.style.cssText;
    layout.forEach(function(name) { canvas.style[name] = look[name]; });
    canvas.style.width = box.width + 'px';
    canvas.style.height = box.height + 'px';
    swapped.push({img: img, canvas: canvas, display: img.style.display});
    img.parentNode.insertBefore(canvas, img);
    img.style.display = 'none';
  }
  function freeze() {
    Array.prototype.slice.call(document.images).forEach(snapshot);
    if (document.getAnimations) {
      document.getAnimations().forEach(function(animation) {
        if (animation.playState !== 'running') return;
        animation.pause();
        paused.push(animation);
      });
    }
    Array.prototype.slice.call(document.querySelectorAll('svg')).forEach(function(svg) {
      if (svg.pauseAnimations) svg.pauseAnimations();
    });
  }
  function thaw() {
    swapped.forEach(function(item) {
      if (item.canvas.parentNode) item.canvas.parentNode.removeChild(item.canvas);
      item.img.style.display = item.display;
    });
    paused.forEach(function(animation) { animation.play(); });
    Array.prototype.slice.call(document.querySelectorAll('svg')).forEach(function(svg) {
      if (svg.unpauseAnimations) svg.unpauseAnimations();
    });
    swapped = [];
    paused = [];
  }
  var frozen = false;
  window.__miraPause = function(pause) {
    if (pause === frozen) return;
    frozen = pause;
    if (pause) freeze(); else thaw();
  };
})();
''';
