(function () {
  var tabs = Array.prototype.slice.call(document.querySelectorAll('[role="tab"]'));
  var panels = tabs.map(function (t) { return document.getElementById(t.getAttribute('aria-controls')); });
  function select(i) {
    tabs.forEach(function (t, k) {
      t.setAttribute('aria-selected', k === i ? 'true' : 'false');
      t.tabIndex = k === i ? 0 : -1;
      panels[k].hidden = k !== i;
    });
  }
  tabs.forEach(function (t, i) {
    t.addEventListener('click', function () { select(i); });
    t.addEventListener('keydown', function (ev) {
      var step = ev.key === 'ArrowRight' ? 1 : ev.key === 'ArrowLeft' ? -1 : 0;
      if (!step) return;
      var n = (i + step + tabs.length) % tabs.length;
      select(n); tabs[n].focus();
    });
  });
  select(0);
  document.querySelectorAll('[data-copy]').forEach(function (b) {
    var label = b.querySelector('span:last-child');
    b.addEventListener('click', function () {
      if (!navigator.clipboard) return;
      navigator.clipboard.writeText(b.getAttribute('data-copy')).then(function () {
        label.textContent = '__COPIED__';
        setTimeout(function () { label.textContent = '__COPY__'; }, 1600);
      });
    });
  });
})();
