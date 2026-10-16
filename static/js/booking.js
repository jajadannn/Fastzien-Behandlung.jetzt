// ===== Booking Calendar =====
(function() {
  const isGuest = document.body.dataset.guestBooking === 'true';
  const confirmationForm = document.getElementById('guest-confirm-form');
  if (confirmationForm) {
    confirmationForm.addEventListener('submit', async function(event) {
      event.preventDefault();
      const button = confirmationForm.querySelector('button');
      const error = document.getElementById('error-msg');
      if (button.disabled) return;
      button.disabled = true; button.textContent = 'Termin wird gebucht …'; error.style.display = 'none';
      try {
        const res = await fetch(confirmationForm.action, {method:'POST', body:new URLSearchParams(new FormData(confirmationForm))});
        const data = await res.json();
        if (!res.ok || !data.success) throw new Error(data.error || 'Bitte wähle einen neuen Termin.');
        confirmationForm.hidden = true;
        const success = document.getElementById('success-msg');
        success.textContent = 'Dein Termin ist verbindlich gebucht. Buchungsnummer: ' + data.appointment_id + '. Die Bestätigung wird per E-Mail übermittelt; bei einer Versandstörung wird der Versand erneut versucht. Für Änderungen oder Absagen erreichst du Thilo telefonisch oder per E-Mail.';
        success.style.display = 'block';
      } catch (err) { error.textContent = err.message || 'Verbindungsfehler. Bitte versuche es erneut.'; error.style.display = 'block'; button.disabled = false; button.textContent = 'Termin kostenpflichtig buchen'; }
    });
    return;
  }
  if (!document.getElementById('calendar-title')) return;
  let requestVersion = 0;
  const detailsForm = document.getElementById('booking-details');
  const homeVisit = document.getElementById('home-visit');
  const price = Number(document.body.dataset.sessionPrice);
  const surcharge = Number(document.body.dataset.surcharge.replace(',','.'));
  const duration = Number(document.body.dataset.duration);
  const credits = Number(document.body.dataset.credits);
  const money = value => new Intl.NumberFormat('de-DE',{style:'currency',currency:'EUR'}).format(value);
  function updatePrice() {
    const total = price + (homeVisit.checked ? surcharge : 0);
    const summary = document.getElementById('booking-price');
    if (summary) summary.textContent = 'Gesamtpreis: ' + money(total) + (credits > 0 ? ' · Eine Sitzung deiner vorhandenen Karte wird verwendet. Abrechnung zum hinterlegten Kartentarif.' : ' · Keine Guthabenverrechnung.');
    const location = document.getElementById('booking-location');
    if(location) location.textContent = homeVisit.checked ? 'Ort: Hausbesuch – ' + (document.body.dataset.homeAddress || 'Bitte Adresse im Profil ergänzen.') : 'Ort: Praxis – ' + document.body.dataset.practiceAddress;
  }
  homeVisit.addEventListener('change',updatePrice);
  updatePrice();
  detailsForm.addEventListener('submit', event => { event.preventDefault(); window.confirmBooking(); });
  if (isGuest) homeVisit.addEventListener('change', () => {
    document.getElementById('guest-address').hidden = !homeVisit.checked;
    ['guest-street','guest-zip','guest-city'].forEach(id => { document.getElementById(id).required = homeVisit.checked; });
  });
  let currentYear, currentMonth, selectedDate = null, selectedSlot = null;

  const berlinDate = new Intl.DateTimeFormat('sv-SE', {timeZone:'Europe/Berlin',year:'numeric',month:'2-digit',day:'2-digit'}).format(new Date());
  const now = new Date(berlinDate + 'T12:00:00');
  currentYear = now.getFullYear();
  currentMonth = now.getMonth();

  const calTitle = document.getElementById('calendar-title');
  const calDays = document.getElementById('calendar-days');
  const slotsTitle = document.getElementById('slots-title');
  const slotsContainer = document.getElementById('slots-container');
  const bookingOptions = document.getElementById('booking-options');

  const months = ['Januar','Februar','März','April','Mai','Juni','Juli','August','September','Oktober','November','Dezember'];

  function renderCalendar() {
    calTitle.textContent = months[currentMonth] + ' ' + currentYear;
    calDays.innerHTML = '';

    const firstDay = new Date(currentYear, currentMonth, 1);
    let startDay = firstDay.getDay();
    if (startDay === 0) startDay = 7; // Monday = 1
    startDay -= 1; // 0-indexed

    const daysInMonth = new Date(currentYear, currentMonth + 1, 0).getDate();
    const today = new Date(berlinDate + 'T00:00:00');
    today.setHours(0,0,0,0);

    document.getElementById('prev-month').disabled = currentYear === now.getFullYear() && currentMonth === now.getMonth();
    const horizon = new Date(today.getTime() + 180 * 86400000);
    document.getElementById('next-month').disabled = currentYear === horizon.getFullYear() && currentMonth === horizon.getMonth();
    // Empty cells before
    for (let i = 0; i < startDay; i++) {
      const d = document.createElement('div');
      d.className = 'cal-day other-month';
      calDays.appendChild(d);
    }

    for (let day = 1; day <= daysInMonth; day++) {
      const date = new Date(currentYear, currentMonth, day);
      const btn = document.createElement('button');
      btn.type = 'button';
      btn.setAttribute('aria-label', day + '. ' + months[currentMonth] + ' ' + currentYear);
      btn.className = 'cal-day';
      btn.setAttribute('aria-pressed',String(!!selectedDate && date.toDateString()===selectedDate.toDateString()));
      btn.dataset.day=String(day);
      btn.textContent = day;

      // Sunday disabled
      if (date.getDay() === 0) {
        btn.classList.add('disabled');
      }
      // Past dates
      if (date < today) {
        btn.classList.add('disabled');
      }
      btn.disabled = date.getDay() === 0 || date < today || date - today > 180 * 86400000;
      btn.classList.toggle('disabled', btn.disabled);
      // Today highlight
      if (date.getTime() === today.getTime()) {
        btn.classList.add('today');
      }
      // Selected
      if (selectedDate && date.toDateString() === selectedDate.toDateString()) {
        btn.classList.add('selected');
      }

      btn.addEventListener('click', async function() {
        selectedDate = new Date(currentYear, currentMonth, day);
        selectedSlot = null;
        renderCalendar();
        calDays.querySelector('[data-day="' + day + '"]').focus();
        await loadSlots(selectedDate);
      });

      calDays.appendChild(btn);
    }
  }

  async function loadSlots(date) {
    const version = ++requestVersion;
    selectedSlot = null;
    const dateStr = date.getFullYear() + '-' +
      String(date.getMonth() + 1).padStart(2, '0') + '-' +
      String(date.getDate()).padStart(2, '0');

    slotsTitle.textContent = date.getDate() + '. ' + months[date.getMonth()] + ' ' + date.getFullYear();
    slotsContainer.textContent = 'Freie Termine werden geladen …';
    slotsContainer.setAttribute('aria-busy', 'true');
    bookingOptions.style.display = 'none';

    try {
      const res = await fetch('/api/appointments/available-slots?date=' + dateStr + '&_=' + Date.now());
      const data = await res.json();
      if (version !== requestVersion) return;
      if (!res.ok) throw new Error(data.error || 'Bitte wähle den Tag erneut.');

      if (data.slots && data.slots.length > 0) {
        slotsContainer.innerHTML = '';
        data.slots.forEach(function(slot) {
          const btn = document.createElement('button');
          btn.type = 'button';
          btn.setAttribute('aria-pressed', 'false');
          btn.className = 'slot-btn';
          btn.textContent = slot.display;
          btn.dataset.time = slot.time;
          btn.addEventListener('click', function() {
            document.querySelectorAll('.slot-btn').forEach(b => { b.classList.remove('selected'); b.setAttribute('aria-pressed','false'); });
            btn.classList.add('selected');
            btn.setAttribute('aria-pressed', 'true');
            document.getElementById('booking-summary').textContent = slotsTitle.textContent + ' · ' + slot.display + ' Uhr';
            selectedSlot = slot.time;
            bookingOptions.style.display = 'block';
          });
          slotsContainer.appendChild(btn);
        });
      } else {
        slotsContainer.innerHTML = '<p class="slots-placeholder">Keine freien Termine an diesem Tag</p>';
      }
    } catch (err) {
      if (version === requestVersion) slotsContainer.textContent = 'Freie Termine konnten nicht geladen werden. Bitte wähle den Tag erneut.';
    } finally {
      if (version === requestVersion) slotsContainer.setAttribute('aria-busy', 'false');
    }
  }

  document.getElementById('prev-month').addEventListener('click', function() {
    currentMonth--;
    if (currentMonth < 0) { currentMonth = 11; currentYear--; }
    renderCalendar();
  });

  document.getElementById('next-month').addEventListener('click', function() {
    currentMonth++;
    if (currentMonth > 11) { currentMonth = 0; currentYear++; }
    renderCalendar();
  });

  renderCalendar();

  // Expose confirmBooking globally
  window.confirmBooking = async function() {
    if (!selectedDate || !selectedSlot || !detailsForm.reportValidity()) return;

    const btn = document.getElementById('confirm-booking');
    if (btn.disabled) return;
    btn.querySelector('.btn-text').style.display = 'none';
    btn.querySelector('.btn-loader').style.display = 'inline-block';
    btn.disabled = true;

    const dateStr = selectedDate.getFullYear() + '-' +
      String(selectedDate.getMonth() + 1).padStart(2, '0') + '-' +
      String(selectedDate.getDate()).padStart(2, '0');

    const errorMsg = document.getElementById('error-msg');
    const successMsg = document.getElementById('success-msg');
    errorMsg.style.display = 'none';
    successMsg.style.display = 'none';

    const payload = {date:dateStr,time:selectedSlot,is_home_visit:homeVisit.checked,expected_total:price+(homeVisit.checked?surcharge:0),expected_duration:duration,expected_using_credit:credits>0,early_start_requested:!!document.getElementById('early-start')?.checked};
    if (isGuest) {
      const value = id => document.getElementById(id).value.trim();
      Object.assign(payload, {first_name:value('guest-first-name'),last_name:value('guest-last-name'),email:value('guest-email'),phone:value('guest-phone'),street:homeVisit.checked ? value('guest-street') : '',zip_code:homeVisit.checked ? value('guest-zip') : '',city:homeVisit.checked ? value('guest-city') : '',source:value('guest-source'),website:value('guest-website')});
    } else payload.notes = document.getElementById('booking-notes').value || null;
    try {
      const res = await fetch(isGuest ? '/api/guest-booking/request' : '/api/appointments/book', {
        method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify(payload)
      });
      const data = await res.json();
      if (res.ok && data.success) {
        successMsg.textContent = isGuest ? data.message : 'Dein Termin ist verbindlich gebucht. Buchungsnummer: ' + data.appointment_id + '. Die Bestätigung wird per E-Mail übermittelt; bei einer Versandstörung wird der Versand erneut versucht.';
        successMsg.style.display = 'block';
        bookingOptions.style.display = 'none';
        selectedSlot = null;
        // Reload slots
        await loadSlots(selectedDate);
        successMsg.scrollIntoView({block:'center',behavior:matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth'});
      } else {
        errorMsg.textContent = data.error || 'Buchung fehlgeschlagen';
        errorMsg.style.display = 'block';
      }
    } catch (err) {
      errorMsg.textContent = 'Verbindungsfehler. Bitte versuche es erneut.';
      errorMsg.style.display = 'block';
    } finally {
      btn.querySelector('.btn-text').style.display = 'inline';
      btn.querySelector('.btn-loader').style.display = 'none';
      btn.disabled = false;
    }
  };
})();
