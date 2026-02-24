var titelname = ["Faszienbehandlung - Thilo Seifried", "Faszienbehandlung Buchen"];
var i = 0;

setInterval(function () {
    document.getElementById("titel").innerHTML = titelname[i];
    i = (i + 1) % titelname.length;
}, 10000);

document.addEventListener("DOMContentLoaded", function () {
    var modal = document.getElementById("appointment-modal");
    var closeButton = document.getElementById("appointment-close");
    var triggers = document.querySelectorAll(".appointment-trigger");

    if (!modal || !closeButton || triggers.length === 0) {
        return;
    }

    function openAppointment(event) {
        if (event) {
            event.preventDefault();
        }
        modal.classList.add("is-open");
        modal.setAttribute("aria-hidden", "false");
    }

    function closeAppointment() {
        modal.classList.remove("is-open");
        modal.setAttribute("aria-hidden", "true");
    }

    triggers.forEach(function (trigger) {
        trigger.addEventListener("click", openAppointment);
    });

    closeButton.addEventListener("click", closeAppointment);

    modal.addEventListener("click", function (event) {
        if (event.target === modal) {
            closeAppointment();
        }
    });

    document.addEventListener("keydown", function (event) {
        if (event.key === "Escape" && modal.classList.contains("is-open")) {
            closeAppointment();
        }
    });
});
