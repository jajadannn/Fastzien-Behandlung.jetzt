// ===== Reveal animations on scroll =====
(function() {
  function applyRevealDelay(selector, stepMs) {
    document.querySelectorAll(selector).forEach(function(el, index) {
      el.style.setProperty('--reveal-delay', (index * stepMs) + 'ms');
    });
  }

  applyRevealDelay('.pain-grid .js-animate', 70);
  applyRevealDelay('.benefit-list .js-animate', 90);
  applyRevealDelay('.steps-row .js-animate', 110);
  applyRevealDelay('.testi-grid .js-animate', 95);
  applyRevealDelay('.price-grid .js-animate', 120);
  applyRevealDelay('.anfahrt-grid .js-animate', 90);
  applyRevealDelay('#ueber .js-animate', 120);

  var animatedEls = document.querySelectorAll('.js-animate');
  if ('IntersectionObserver' in window && animatedEls.length && !window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
    animatedEls.forEach(function(el) { el.classList.add('hidden'); });
    var revealObserver = new IntersectionObserver(
      function(entries, observer) {
        entries.forEach(function(entry) {
          if (!entry.isIntersecting) return;
          entry.target.classList.remove('hidden');
          observer.unobserve(entry.target);
        });
      },
      { threshold: 0.12, rootMargin: '0px 0px -10%' }
    );
    animatedEls.forEach(function(el) { revealObserver.observe(el); });
  }

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
