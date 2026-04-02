// ===== Admin Dashboard JS =====

async function logout() {
  await fetch('/api/auth/logout', { method: 'POST' });
  window.location.href = '/';
}

async function markPaid(paymentId) {
  if (!confirm('Zahlung als bezahlt markieren?')) return;
  try {
    const res = await fetch('/api/admin/payments/' + paymentId + '/mark-paid', { method: 'POST' });
    const data = await res.json();
    if (data.success) location.reload();
    else alert(data.error || 'Fehler');
  } catch (err) {
    alert('Verbindungsfehler');
  }
}

async function adminCancelAppointment(id) {
  if (!confirm('Termin stornieren?')) return;
  try {
    const res = await fetch('/api/appointments/' + id + '/cancel', { method: 'POST' });
    const data = await res.json();
    if (data.success) location.reload();
    else alert(data.error || 'Fehler');
  } catch (err) {
    alert('Verbindungsfehler');
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
