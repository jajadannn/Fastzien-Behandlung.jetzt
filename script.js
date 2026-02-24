const SITE_TITLE = "Faszienbehandlung in Koblenz – Thilo Seifried";

// TODO: Diesen Link mit deinem öffentlichen Nextcloud-Appointments-Link ersetzen.
const NEXTCLOUD_APPOINTMENT_URL = "https://cloud.sdlv.de/apps/calendar/appointment/aRm9fmasaa5x";
const IFRAME_FALLBACK_DELAY_MS = 2500;

document.title = SITE_TITLE;

document.addEventListener("DOMContentLoaded", function () {
    const modal = document.getElementById("appointment-modal");
    const iframe = document.getElementById("appointment-iframe");
    const closeButton = document.getElementById("appointment-close");
    const triggers = document.querySelectorAll(".book-trigger");
    const directLink = document.getElementById("appointment-direct-link");
    const helpText = document.getElementById("appointment-help");

    let iframeLoaded = false;
    let fallbackTimer;

    function resetFallback() {
        iframeLoaded = false;
        helpText.classList.remove("is-visible");
        directLink.classList.remove("is-visible");
        clearTimeout(fallbackTimer);

        fallbackTimer = setTimeout(function () {
            if (!iframeLoaded) {
                helpText.classList.add("is-visible");
                directLink.classList.add("is-visible");
            }
        }, IFRAME_FALLBACK_DELAY_MS);
    }

    function openAppointmentModal() {
        directLink.href = NEXTCLOUD_APPOINTMENT_URL;
        iframe.src = NEXTCLOUD_APPOINTMENT_URL;
        resetFallback();

        modal.classList.add("is-open");
        modal.setAttribute("aria-hidden", "false");
        document.body.classList.add("modal-open");
    }

    function closeAppointmentModal() {
        modal.classList.remove("is-open");
        modal.setAttribute("aria-hidden", "true");
        iframe.removeAttribute("src");
        document.body.classList.remove("modal-open");
        clearTimeout(fallbackTimer);
    }

    iframe.addEventListener("load", function () {
        iframeLoaded = true;
        helpText.classList.remove("is-visible");
        directLink.classList.remove("is-visible");
        clearTimeout(fallbackTimer);
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
