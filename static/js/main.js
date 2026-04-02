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
  if ('IntersectionObserver' in window && animatedEls.length) {
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

  // Keep mail address out of plain source spam scrapers
  var EMAIL = ['termin', 'faszienbehandlung.jetzt'];
  var mail = EMAIL[0] + '@' + EMAIL[1];
  document.querySelectorAll('.js-email').forEach(function(el) {
    el.textContent = mail;
    el.setAttribute('href', 'mailto:' + mail);
    el.setAttribute('aria-label', 'E-Mail an ' + mail);
  });
})();
