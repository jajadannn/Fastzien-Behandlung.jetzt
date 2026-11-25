// Google Ads conversion tag, loaded only after the visitor allows it (§ 25 TDDDG).
// The choice is kept in localStorage; "Cookie-Einstellungen" in the footer asks again.
(function () {
  var ID = 'AW-18507567266', KEY = 'faszien-consent';
  var loaded = false;

  function load() {
    if (loaded) return;
    loaded = true;
    window.dataLayer = window.dataLayer || [];
    window.gtag = function () { window.dataLayer.push(arguments); };
    // Consent Mode v2 (Google's EU user consent policy): denied until the visitor allows all.
    window.gtag('consent', 'default', { ad_storage: 'denied', ad_user_data: 'denied', ad_personalization: 'denied', analytics_storage: 'denied' });
    window.gtag('consent', 'update', { ad_storage: 'granted', ad_user_data: 'granted', ad_personalization: 'granted', analytics_storage: 'granted' });
    window.gtag('js', new Date());
    window.gtag('config', ID);
    var s = document.createElement('script');
    s.async = true;
    s.src = 'https://www.googletagmanager.com/gtag/js?id=' + ID;
    document.head.appendChild(s);
  }

  function choice() { try { return localStorage.getItem(KEY); } catch (e) { return null; } }
  function remember(v) { try { localStorage.setItem(KEY, v); } catch (e) { /* private mode: ask again next visit */ } }

  function banner() {
    if (document.getElementById('consent-banner')) return;
    var box = document.createElement('div');
    box.id = 'consent-banner';
    box.className = 'consent-banner';
    box.setAttribute('role', 'dialog');
    box.setAttribute('aria-label', 'Einwilligung zu Google Ads');
    box.innerHTML = '<p>Darf Google Ads Cookies setzen? Damit messe ich, ob Besucher über eine Anzeige einen Termin buchen, und Google kann dir passende Anzeigen zeigen (Remarketing). Google erhält dafür Daten über deinen Besuch. '
      + '<a href="/datenschutz#dp-4">Mehr dazu</a></p>'
      + '<div class="consent-actions"><button type="button" data-consent="denied">Ablehnen</button>'
      + '<button type="button" data-consent="granted">Erlauben</button></div>';
    box.addEventListener('click', function (e) {
      var v = e.target.getAttribute && e.target.getAttribute('data-consent');
      if (!v) return;
      remember(v);
      box.remove();
      if (v === 'granted') load();
      else if (loaded) window.gtag('consent', 'update', { ad_storage: 'denied', ad_user_data: 'denied', ad_personalization: 'denied', analytics_storage: 'denied' });
    });
    document.body.appendChild(box);
  }

  // Called by booking.js after a confirmed booking; does nothing without consent.
  window.faszienConversion = function () {
    if (loaded) window.gtag('event', 'conversion', { send_to: ID + '/Zje9CKytxL8dEKKhjPlE' });
  };

  function start() {
    var c = choice();
    if (c === 'granted') load();
    else if (c !== 'denied') banner();
    document.addEventListener('click', function (e) {
      if (!e.target.closest || !e.target.closest('[data-consent-settings]')) return;
      e.preventDefault();
      banner();
    });
  }
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', start); else start();
})();
