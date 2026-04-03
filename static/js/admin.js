// ===== Admin Dashboard JS =====

async function logout() {
  await fetch('/api/auth/logout', { method: 'POST' });
  window.location.href = '/';
}

async function markPaid(paymentId) {
  try {
    const res = await fetch('/api/admin/payments/' + paymentId + '/mark-paid', { method: 'POST' });
    const data = await res.json();
    if (data.success) {
      window.location.href = window.location.pathname + '?t=' + Date.now();
    } else {
      alert(data.error || 'Fehler');
    }
  } catch (err) {
    alert('Verbindungsfehler');
  }
}

function openCancelModal(id) {
  document.getElementById('cancel-appointment-id').value = id;
  document.getElementById('cancel-suggest-1').value = '';
  document.getElementById('cancel-suggest-2').value = '';
  document.getElementById('cancel-modal-overlay').classList.add('active');
}

function closeCancelModal() {
  document.getElementById('cancel-modal-overlay').classList.remove('active');
}

async function confirmCancelWithSuggestions() {
  const id = document.getElementById('cancel-appointment-id').value;
  const s1 = document.getElementById('cancel-suggest-1').value;
  const s2 = document.getElementById('cancel-suggest-2').value;
  
  const slots = [];
  if (s1) {
    const d1 = new Date(s1);
    slots.push(d1.toLocaleDateString('de-DE', { weekday: 'long', day: 'numeric', month: 'long', year: 'numeric', hour: '2-digit', minute: '2-digit' }));
  }
  if (s2) {
    const d2 = new Date(s2);
    slots.push(d2.toLocaleDateString('de-DE', { weekday: 'long', day: 'numeric', month: 'long', year: 'numeric', hour: '2-digit', minute: '2-digit' }));
  }

  const btn = document.getElementById('btn-cancel-confirm');
  btn.querySelector('.btn-text').style.display = 'none';
  btn.querySelector('.btn-loader').style.display = 'inline-block';
  btn.disabled = true;

  try {
    const res = await fetch('/api/admin/appointments/cancel-suggest', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ appointment_id: parseInt(id), slots: slots }),
    });
    const data = await res.json();
    if (data.success) {
      window.location.href = window.location.pathname + '?t=' + Date.now();
    } else {
      alert(data.error || 'Fehler beim Stornieren');
      closeCancelModal();
    }
  } catch (err) {
    alert('Verbindungsfehler');
    closeCancelModal();
  } finally {
    btn.querySelector('.btn-text').style.display = 'inline';
    btn.querySelector('.btn-loader').style.display = 'none';
    btn.disabled = false;
  }
}

function addSuggestSlot() {
  const container = document.getElementById('suggest-slots');
  const row = document.createElement('div');
  row.className = 'suggest-slot-row';
  row.innerHTML = '<input type="datetime-local" class="suggest-input"><button onclick="this.parentElement.remove()" class="btn-small btn-danger">✕</button>';
  container.appendChild(row);
}

async function sendSuggestion(customerId) {
  const inputs = document.querySelectorAll('.suggest-input');
  const slots = [];
  inputs.forEach(function(input) {
    if (input.value) {
      const d = new Date(input.value);
      slots.push(d.toLocaleDateString('de-DE', { weekday: 'long', day: 'numeric', month: 'long', year: 'numeric', hour: '2-digit', minute: '2-digit' }));
    }
  });

  if (slots.length === 0) {
    alert('Bitte mindestens einen Termin auswählen');
    return;
  }

  try {
    const res = await fetch('/api/admin/appointments/suggest', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ customer_id: customerId, slots: slots }),
    });
    const data = await res.json();
    if (data.success) {
      alert('Terminvorschläge per E-Mail gesendet!');
    } else {
      alert(data.error || 'Fehler');
    }
  } catch (err) {
    alert('Verbindungsfehler');
  }
}
