// relaxng-schema-rustdoc-theme.js — runtime behaviour for the
// rustdoc-styled schema-docs theme used by latexml-oxide's
// `--schemadocs` post-pass. Co-evolves with the sister stylesheet
// `resources/CSS/relaxng-schema-rustdoc-theme.css`.
//
// Three pieces of behaviour live here:
//
// 1. Pre-paint theme + display preference application —
//    reads `localStorage` and stamps `data-theme` / `data-pref-*`
//    on `<html>` BEFORE first paint so the CSS lights up the
//    right palette without flashing the wrong one.
//
// 2. Settings popover wiring — once the DOM is ready, attaches
//    `change` handlers to the Theme radios and Display checkboxes
//    inside `<details data-schema-theme-widget>`, plus a
//    `prefers-color-scheme` listener that re-resolves theme when
//    `system` is selected and a click-outside handler to dismiss
//    the popover.
//
// 3. In-page filter — on a long definition list (≥ 25 items) on a
//    schema-def page, inserts a sticky search input that toggles
//    `display: none` on non-matching `<dt>`/`<dd>` pairs. Items
//    default to visible so browser Ctrl-F still works.
//
// The script tag is injected by `inject_theme_switcher` in
// `latexml_post::schema_docs` as a *non-deferred* `<script src>` in
// `<head>`, so applyTheme() runs synchronously before the body is
// parsed (no flash of wrong palette). The post-DOM wiring waits on
// `DOMContentLoaded`.

(function () {
  'use strict';

  // ---- 1. Pre-paint theme + display preference application -----------------

  // Stamp `data-theme` + `data-theme-pref` + `data-pref-*` on <html>
  // from localStorage. Wrapped in try/catch because localStorage can
  // throw in privacy-mode browsers; failures must not block page render.
  function applyTheme() {
    try {
      var pref = localStorage.getItem('schema-theme') || 'system';
      var theme = pref === 'system'
        ? (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
        : pref;
      var root = document.documentElement;
      root.setAttribute('data-theme', theme);
      root.setAttribute('data-theme-pref', pref);
      ['sidebar'].forEach(function (key) {
        if (localStorage.getItem('schema-pref-' + key) === 'on') {
          root.setAttribute('data-pref-' + key, 'on');
        }
      });
    } catch (e) {}
  }

  // ---- 2. Settings popover wiring ------------------------------------------

  // Attach change handlers to the Theme radios + Display checkboxes,
  // plus a system colour-scheme matchMedia listener and a
  // click-outside dismissal for the popover.
  function wireSettings() {
    var root = document.documentElement;
    var pref = root.getAttribute('data-theme-pref') || 'system';

    // Theme radios.
    var radios = document.querySelectorAll('input[name="schema-theme-radio"]');
    for (var i = 0; i < radios.length; i++) {
      if (radios[i].value === pref) radios[i].checked = true;
      radios[i].addEventListener('change', function (e) {
        var v = e.target.value;
        try { localStorage.setItem('schema-theme', v); } catch (_) {}
        var t = v === 'system'
          ? (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light')
          : v;
        root.setAttribute('data-theme', t);
        root.setAttribute('data-theme-pref', v);
      });
    }

    // Display preference checkboxes (sans-serif fonts / hide sidebar /
    // wrap content models). Each flips `data-pref-<key>` and persists.
    var checks = document.querySelectorAll('input[type="checkbox"][data-schema-pref]');
    for (var j = 0; j < checks.length; j++) {
      var key = checks[j].getAttribute('data-schema-pref');
      if (root.getAttribute('data-pref-' + key) === 'on') checks[j].checked = true;
      checks[j].addEventListener('change', function (e) {
        var k = e.target.getAttribute('data-schema-pref');
        var attr = 'data-pref-' + k;
        if (e.target.checked) {
          try { localStorage.setItem('schema-pref-' + k, 'on'); } catch (_) {}
          root.setAttribute(attr, 'on');
        } else {
          try { localStorage.removeItem('schema-pref-' + k); } catch (_) {}
          root.removeAttribute(attr);
        }
      });
    }

    // Re-resolve theme when the system colour-scheme changes — only
    // when the user has chosen `system` (otherwise their explicit
    // choice wins).
    if (window.matchMedia) {
      matchMedia('(prefers-color-scheme: dark)').addEventListener('change', function (e) {
        if (root.getAttribute('data-theme-pref') === 'system') {
          root.setAttribute('data-theme', e.matches ? 'dark' : 'light');
        }
      });
    }

    // Click-outside dismissal for the popover.
    document.addEventListener('click', function (e) {
      var d = document.querySelector('details[data-schema-theme-widget]');
      if (d && d.open && !d.contains(e.target)) d.removeAttribute('open');
    });
  }

  // ---- 3. In-page schema-def filter ----------------------------------------

  // On long definition-list pages (≥ 25 schema-def items), insert a
  // sticky search input above the list. Typing applies
  // `display: none` (via `.schema-filter-hidden`) to non-matching
  // `<dt>`/`<dd>` pairs. Items default to visible, so browser Ctrl-F
  // still finds anything in the DOM.
  function wireFilter() {
    var dl = document.querySelector('dl.ltx_description');
    if (!dl) return;
    var dts = dl.querySelectorAll(':scope > dt.schema-def');
    if (dts.length < 25) return;

    var wrap = document.createElement('div');
    wrap.className = 'schema-filter';
    var input = document.createElement('input');
    input.type = 'search';
    input.placeholder = 'Filter by name… (' + dts.length + ' items)';
    input.setAttribute('aria-label', 'Filter schema definitions');
    var count = document.createElement('span');
    count.className = 'schema-filter-count';
    wrap.appendChild(input);
    wrap.appendChild(count);
    dl.parentNode.insertBefore(wrap, dl);

    var items = Array.prototype.map.call(dts, function (dt) {
      var nameEl = dt.querySelector('.schema-name');
      var name = nameEl ? nameEl.textContent.toLowerCase() : '';
      var dd = dt.nextElementSibling;
      if (dd && dd.tagName !== 'DD') dd = null;
      return { dt: dt, dd: dd, name: name };
    });

    function apply(query) {
      var q = (query || '').trim().toLowerCase();
      var visible = 0;
      items.forEach(function (it) {
        var match = !q || it.name.indexOf(q) !== -1;
        if (match) {
          it.dt.classList.remove('schema-filter-hidden');
          if (it.dd) it.dd.classList.remove('schema-filter-hidden');
          visible++;
        } else {
          it.dt.classList.add('schema-filter-hidden');
          if (it.dd) it.dd.classList.add('schema-filter-hidden');
        }
      });
      count.textContent = q ? (visible + ' / ' + items.length) : '';
    }

    input.addEventListener('input', function (e) { apply(e.target.value); });
  }

  // ---- Boot order ----------------------------------------------------------

  // applyTheme() runs synchronously before paint (the script tag in
  // <head> is non-deferred, so HTML parsing is blocked while we
  // execute). The post-DOM wiring waits on DOMContentLoaded so the
  // widget markup is in place.
  applyTheme();
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', function () {
      wireSettings();
      wireFilter();
    });
  } else {
    wireSettings();
    wireFilter();
  }
})();
