const titelname = ["Faszienbehandlung - Thilo Seifried", "Faszienbehandlung Buchen"];
let i = 0;

setInterval(function () {
    document.getElementById("titel").textContent = titelname[i];
    i = (i + 1) % titelname.length;
}, 10000);

// TODO: Diesen Link mit deinem öffentlichen Nextcloud-Appointments-Link ersetzen.
const NEXTCLOUD_APPOINTMENT_URL = "https://cloud.sdlv.de/apps/calendar/appointment/aRm9fmasaa5x";

document.addEventListener("DOMContentLoaded", function () {
    const modal = document.getElementById("appointment-modal");
    const iframe = document.getElementById("appointment-iframe");
    const closeButton = document.getElementById("appointment-close");
    const triggers = document.querySelectorAll(".book-trigger");

    function openAppointmentModal() {
        iframe.src = NEXTCLOUD_APPOINTMENT_URL;
        modal.classList.add("is-open");
        modal.setAttribute("aria-hidden", "false");
        document.body.classList.add("modal-open");
    }

    function closeAppointmentModal() {
        modal.classList.remove("is-open");
        modal.setAttribute("aria-hidden", "true");
        iframe.removeAttribute("src");
        document.body.classList.remove("modal-open");
    }

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
