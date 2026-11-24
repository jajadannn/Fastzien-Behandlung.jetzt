// Content is shown at rest: no scroll-triggered reveal (design review, November 2026).
(function() {
  // ===== Mobile nav toggle =====
  var navToggle = document.querySelector('.nav-toggle');
  var navMenu = document.getElementById('topbar-menu');
  if (navToggle && navMenu) {
    var setNavOpen = function(open) {
      navMenu.classList.toggle('is-open', open);
      navToggle.setAttribute('aria-expanded', String(open));
      navToggle.setAttribute('aria-label', open ? 'Menü schließen' : 'Menü öffnen');
    };
    navToggle.addEventListener('click', function() {
      setNavOpen(!navMenu.classList.contains('is-open'));
    });
    navMenu.querySelectorAll('a').forEach(function(link) {
      link.addEventListener('click', function() { setNavOpen(false); });
    });
    document.addEventListener('keydown', function(e) {
      if (e.key === 'Escape' && navMenu.classList.contains('is-open')) {
        setNavOpen(false);
        navToggle.focus();
      }
    });
    document.addEventListener('click', function(e) {
      if (!navMenu.classList.contains('is-open')) return;
      if (navMenu.contains(e.target) || navToggle.contains(e.target)) return;
      setNavOpen(false);
    });
  }
})();
