const titleNames = [
    "Faszienbehandlung in Dortmund - Thilo Seifried",
    "Termin buchen | Faszienbehandlung Dortmund"
];
let titleIndex = 0;

function updateTitle() {
    document.title = titleNames[titleIndex];
    titleIndex = (titleIndex + 1) % titleNames.length;
}

updateTitle();
setInterval(updateTitle, 10000);

const NEXTCLOUD_APPOINTMENT_URL = "https://cloud.sdlv.de/apps/calendar/appointment/aRm9fmasaa5x";

document.addEventListener("DOMContentLoaded", function () {
    const modal = document.getElementById("appointment-modal");
    const iframe = document.getElementById("appointment-iframe");
    const closeButton = document.getElementById("appointment-close");
    const statusText = document.getElementById("appointment-status");
    const fallbackLink = document.getElementById("appointment-fallback-link");
    const triggers = document.querySelectorAll(".book-trigger");

    let fallbackTimeout;

    function showFallback() {
        statusText.hidden = false;
        fallbackLink.hidden = false;
    }

    function hideFallback() {
        statusText.hidden = true;
        fallbackLink.hidden = true;
    }

    function openAppointmentModal() {
        hideFallback();
        iframe.src = NEXTCLOUD_APPOINTMENT_URL;
        fallbackTimeout = setTimeout(showFallback, 2500);
        modal.classList.add("is-open");
        modal.setAttribute("aria-hidden", "false");
        document.body.classList.add("modal-open");
    }

    function closeAppointmentModal() {
        clearTimeout(fallbackTimeout);
        modal.classList.remove("is-open");
        modal.setAttribute("aria-hidden", "true");
        iframe.removeAttribute("src");
        document.body.classList.remove("modal-open");
    }

    iframe.addEventListener("load", function () {
        clearTimeout(fallbackTimeout);
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
