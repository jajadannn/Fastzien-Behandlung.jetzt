const NEXTCLOUD_APPOINTMENT_URL = "https://cloud.sdlv.de/apps/calendar/appointment/aRm9fmasaa5x";

document.addEventListener("DOMContentLoaded", function () {
    const modal = document.getElementById("appointment-modal");
    const iframe = document.getElementById("appointment-iframe");
    const closeButton = document.getElementById("appointment-close");
    const triggers = document.querySelectorAll(".book-trigger");
    const appointmentHint = document.getElementById("appointment-hint");
    const directLink = document.getElementById("appointment-direct-link");

    let iframeFallbackTimeout;

    directLink.href = NEXTCLOUD_APPOINTMENT_URL;

    function showIframeFallback() {
        appointmentHint.hidden = false;
        iframe.setAttribute("hidden", "true");
    }

    function resetFallbackState() {
        appointmentHint.hidden = true;
        iframe.removeAttribute("hidden");
    }

    function openAppointmentModal() {
        resetFallbackState();
        iframe.src = NEXTCLOUD_APPOINTMENT_URL;
        modal.classList.add("is-open");
        modal.setAttribute("aria-hidden", "false");
        document.body.classList.add("modal-open");

        // Wenn die externe Seite iFrame-Einbettung ablehnt (X-Frame-Options/CSP),
        // bleibt der Frame oft leer. Dann zeigen wir nach kurzer Zeit den Direktlink.
        iframeFallbackTimeout = window.setTimeout(showIframeFallback, 2500);
    }

    function closeAppointmentModal() {
        window.clearTimeout(iframeFallbackTimeout);
        modal.classList.remove("is-open");
        modal.setAttribute("aria-hidden", "true");
        iframe.removeAttribute("src");
        document.body.classList.remove("modal-open");
        resetFallbackState();
    }

    iframe.addEventListener("load", function () {
        window.clearTimeout(iframeFallbackTimeout);
    });

    triggers.forEach((trigger) => {
        trigger.addEventListener("click", openAppointmentModal);
    });

    closeButton.addEventListener("click", closeAppointmentModal);

    modal.addEventListener("click", function (event) {
        if (event.target === modal) {
            closeAppointmentModal();
        }
    });

    document.addEventListener("keydown", function (event) {
        if (event.key === "Escape" && modal.classList.contains("is-open")) {
            closeAppointmentModal();
        }
    });
});
