(function () {
  'use strict';

  var STORAGE_KEY = 'ikb-theme';
  var DARK_CSS_ID = 'jtd-dark-css';

  function getPreferredTheme() {
    var stored = localStorage.getItem(STORAGE_KEY);
    if (stored === 'dark' || stored === 'light') return stored;
    if (window.matchMedia('(prefers-color-scheme: dark)').matches) return 'dark';
    return 'light';
  }

  function loadDarkCSS() {
    if (document.getElementById(DARK_CSS_ID)) return;
    var link = document.createElement('link');
    link.rel = 'stylesheet';
    link.href = document.querySelector('link[href*="just-the-docs-default"]').href.replace('default', 'dark');
    link.id = DARK_CSS_ID;
    document.head.appendChild(link);
  }

  function setTheme(theme) {
    var darkLink = document.getElementById(DARK_CSS_ID);
    if (theme === 'dark') {
      loadDarkCSS();
      var link = document.getElementById(DARK_CSS_ID);
      if (link) link.disabled = false;
    } else if (darkLink) {
      darkLink.disabled = true;
    }
    localStorage.setItem(STORAGE_KEY, theme);
    updateButtonIcon(theme);
  }

  function toggleTheme() {
    var isDark = !document.getElementById(DARK_CSS_ID) || document.getElementById(DARK_CSS_ID).disabled;
    setTheme(isDark ? 'dark' : 'light');
  }

  function updateButtonIcon(theme) {
    var btn = document.getElementById('ikb-theme-toggle');
    if (!btn) return;
    btn.innerHTML = theme === 'dark'
      ? '<svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M12 7c-2.76 0-5 2.24-5 5s2.24 5 5 5 5-2.24 5-5-2.24-5-5-5zM2 13h2c.55 0 1-.45 1-1s-.45-1-1-1H2c-.55 0-1 .45-1 1s.45 1 1 1zm18 0h2c.55 0 1-.45 1-1s-.45-1-1-1h-2c-.55 0-1 .45-1 1s.45 1 1 1zM11 2v2c0 .55.45 1 1 1s1-.45 1-1V2c0-.55-.45-1-1-1s-1 .45-1 1zm0 18v2c0 .55.45 1 1 1s1-.45 1-1v-2c0-.55-.45-1-1-1s-1 .45-1 1zM5.99 4.58a.996.996 0 00-1.41 0 .996.996 0 000 1.41l1.06 1.06c.39.39 1.03.39 1.41 0s.39-1.03 0-1.41L5.99 4.58zm12.37 12.37a.996.996 0 00-1.41 0 .996.996 0 000 1.41l1.06 1.06c.39.39 1.03.39 1.41 0a.996.996 0 000-1.41l-1.06-1.06zm1.06-10.96a.996.996 0 000-1.41.996.996 0 00-1.41 0l-1.06 1.06c-.39.39-.39 1.03 0 1.41s1.03.39 1.41 0l1.06-1.06zM7.05 18.36a.996.996 0 000-1.41.996.996 0 00-1.41 0l-1.06 1.06c-.39.39-.39 1.03 0 1.41s1.03.39 1.41 0l1.06-1.06z"></path></svg>'
      : '<svg viewBox="0 0 24 24" width="18" height="18"><path fill="currentColor" d="M12 3a9 9 0 109 9c0-.46-.04-.92-.1-1.36a5.389 5.389 0 01-4.4 2.26 5.403 5.403 0 01-3.14-9.8c-.44-.06-.9-.1-1.36-.1z"></path></svg>';
  }

  function injectToggleButton() {
    var aux = document.querySelector('.aux-nav');
    if (!aux) return;
    var btn = document.createElement('button');
    btn.id = 'ikb-theme-toggle';
    btn.setAttribute('aria-label', 'Toggle dark/light mode');
    btn.style.cssText = 'background:none;border:none;cursor:pointer;color:inherit;padding:0.5rem;display:flex;align-items:center;';
    btn.addEventListener('click', toggleTheme);
    var list = aux.querySelector('ul');
    if (list) {
      var item = document.createElement('li');
      item.className = 'aux-nav-list-item';
      item.appendChild(btn);
      list.appendChild(item);
    }
    var preferred = getPreferredTheme();
    updateButtonIcon(preferred);
    if (preferred === 'dark') {
      loadDarkCSS();
    }
  }

  function buildTOC() {
    var content = document.querySelector('.main-content');
    if (!content) return;

    var headings = content.querySelectorAll('h2, h3');
    if (headings.length < 2) return;

    var nav = document.createElement('nav');
    nav.className = 'ikb-toc';
    nav.setAttribute('aria-label', 'Page table of contents');
    var title = document.createElement('div');
    title.className = 'ikb-toc-title';
    title.textContent = 'On this page';
    nav.appendChild(title);

    var list = document.createElement('ul');
    nav.appendChild(list);

    function addLink(h, sub) {
      if (!h.id) return;
      var li = document.createElement('li');
      var a = document.createElement('a');
      a.href = '#' + h.id;
      a.textContent = h.textContent;
      if (sub) li.className = 'ikb-toc-sub';
      li.appendChild(a);
      list.appendChild(li);
    }

    headings.forEach(function (h) {
      addLink(h, h.tagName === 'H3');
    });

    document.body.appendChild(nav);
  }

  if (document.readyState === 'complete' || document.readyState === 'interactive') {
    injectToggleButton();
    buildTOC();
  } else {
    document.addEventListener('DOMContentLoaded', function () {
      injectToggleButton();
      buildTOC();
    });
  }
})();
