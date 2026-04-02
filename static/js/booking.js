// ===== Booking Calendar =====
(function() {
  let currentYear, currentMonth, selectedDate = null, selectedSlot = null;

  const now = new Date();
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
    const today = new Date();
    today.setHours(0,0,0,0);

    // Empty cells before
    for (let i = 0; i < startDay; i++) {
      const d = document.createElement('div');
      d.className = 'cal-day other-month';
      calDays.appendChild(d);
    }

    for (let day = 1; day <= daysInMonth; day++) {
      const date = new Date(currentYear, currentMonth, day);
      const btn = document.createElement('button');
      btn.className = 'cal-day';
      btn.textContent = day;

      // Sunday disabled
      if (date.getDay() === 0) {
        btn.classList.add('disabled');
      }
      // Past dates
      if (date < today) {
        btn.classList.add('disabled');
      }
      // Today highlight
      if (date.getTime() === today.getTime()) {
        btn.classList.add('today');
      }
      // Selected
      if (selectedDate && date.toDateString() === selectedDate.toDateString()) {
        btn.classList.add('selected');
      }

      btn.addEventListener('click', function() {
        selectedDate = new Date(currentYear, currentMonth, day);
        selectedSlot = null;
        renderCalendar();
        loadSlots(selectedDate);
      });

      calDays.appendChild(btn);
    }
  }

  async function loadSlots(date) {
    const dateStr = date.getFullYear() + '-' +
      String(date.getMonth() + 1).padStart(2, '0') + '-' +
      String(date.getDate()).padStart(2, '0');

    slotsTitle.textContent = date.getDate() + '. ' + months[date.getMonth()] + ' ' + date.getFullYear();
    slotsContainer.innerHTML = '<div class="slots-loading">Laden...</div>';
    bookingOptions.style.display = 'none';

    try {
      const res = await fetch('/api/appointments/available-slots?date=' + dateStr);
      const data = await res.json();

      if (data.slots && data.slots.length > 0) {
        slotsContainer.innerHTML = '';
        data.slots.forEach(function(slot) {
          const btn = document.createElement('button');
          btn.className = 'slot-btn';
          btn.textContent = '🕐 ' + slot.display;
          btn.dataset.time = slot.time;
          btn.addEventListener('click', function() {
            document.querySelectorAll('.slot-btn').forEach(b => b.classList.remove('selected'));
            btn.classList.add('selected');
            selectedSlot = slot.time;
            bookingOptions.style.display = 'block';
          });
          slotsContainer.appendChild(btn);
        });
      } else {
        slotsContainer.innerHTML = '<p class="slots-placeholder">Keine freien Termine an diesem Tag</p>';
      }
    } catch (err) {
      slotsContainer.innerHTML = '<p class="slots-placeholder">Fehler beim Laden der Termine</p>';
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
    if (!selectedDate || !selectedSlot) return;

    const btn = document.getElementById('confirm-booking');
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

    try {
      const res = await fetch('/api/appointments/book', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          date: dateStr,
          time: selectedSlot,
          is_home_visit: document.getElementById('home-visit').checked,
          notes: document.getElementById('booking-notes').value || null,
        }),
      });
      const data = await res.json();
      if (data.success) {
        successMsg.textContent = '✓ Termin erfolgreich gebucht! Du erhältst eine Bestätigung per E-Mail.';
        successMsg.style.display = 'block';
        bookingOptions.style.display = 'none';
        selectedSlot = null;
        // Reload slots
        loadSlots(selectedDate);
        window.scrollTo({ top: 0, behavior: 'smooth' });
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
