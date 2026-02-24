var titelname = ["Faszienbehandlung - Thilo Seifried", "Faszienbehandlung Buchen"];
var i = 0;

setInterval(function () {
    document.getElementById("titel").innerHTML = titelname[i];
    i = (i + 1) % titelname.length;
}, 10000);

document.addEventListener("DOMContentLoaded", function () {
    var modal = document.getElementById("appointment-modal");
    var openButtons = document.querySelectorAll(".appointment-trigger");
    var closeButtons = document.querySelectorAll("[data-close-modal]");

    function openModal() {
        modal.classList.add("is-open");
        modal.setAttribute("aria-hidden", "false");
        document.body.classList.add("modal-open");
    }

    function closeModal() {
        modal.classList.remove("is-open");
        modal.setAttribute("aria-hidden", "true");
        document.body.classList.remove("modal-open");
    }

    openButtons.forEach(function (button) {
        button.addEventListener("click", openModal);
    });

    closeButtons.forEach(function (button) {
        button.addEventListener("click", closeModal);
    });

    document.addEventListener("keydown", function (event) {
        if (event.key === "Escape" && modal.classList.contains("is-open")) {
            closeModal();
        }
    });
});
