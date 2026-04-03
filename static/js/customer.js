// ===== Customer Portal JS =====


async function cancelAppointment(id) {
  try {
    const res = await fetch('/api/appointments/' + id + '/cancel', { method: 'POST' });
    const data = await res.json();
    if (data.success) {
      window.location.href = window.location.pathname + '?t=' + Date.now();
    } else {
      alert(data.error || 'Stornierung fehlgeschlagen');
    }
  } catch (err) {
    alert('Verbindungsfehler');
  }
}
