(function() {
  'use strict';

  var STORAGE_KEY = 'ikb-dark-mode';
  var ATTR = 'data-skin';

  function getPreferredTheme() {
    var saved = localStorage.getItem(STORAGE_KEY);
    if (saved) return saved;
    if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
      return 'dark';
    }
    return 'default';
  }

  function setTheme(skin) {
    document.documentElement.setAttribute(ATTR, skin);
    localStorage.setItem(STORAGE_KEY, skin);
  }

  function toggleTheme() {
    var current = document.documentElement.getAttribute(ATTR) || 'default';
    setTheme(current === 'dark' ? 'default' : 'dark');
  }

  function createToggleButton() {
    var btn = document.createElement('button');
    btn.className = 'dark-mode-toggle';
    btn.setAttribute('aria-label', '切换暗黑/明亮模式');
    btn.innerHTML = '<span class="icon">&#9790;</span>';
    btn.addEventListener('click', toggleTheme);
    return btn;
  }

  function injectToggle() {
    var nav = document.querySelector('.greedy-nav');
    if (!nav) return;
    var btn = createToggleButton();
    nav.appendChild(btn);
  }

  // Initialize
  setTheme(getPreferredTheme());

  // Insert toggle after DOM ready
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', injectToggle);
  } else {
    injectToggle();
  }
})();
