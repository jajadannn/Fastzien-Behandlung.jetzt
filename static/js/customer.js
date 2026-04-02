// ===== Customer Portal JS =====

async function logout() {
  await fetch('/api/auth/logout', { method: 'POST' });
  window.location.href = '/';
}

async function cancelAppointment(id) {
  if (!confirm('Termin wirklich stornieren?')) return;
  try {
    const res = await fetch('/api/appointments/' + id + '/cancel', { method: 'POST' });
    const data = await res.json();
    if (data.success) {
      location.reload();
    } else {
      alert(data.error || 'Stornierung fehlgeschlagen');
    }
  } catch (err) {
    alert('Verbindungsfehler');
  }
}
